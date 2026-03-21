// SPDX-License-Identifier: GPL-2.0

#![allow(missing_docs)]

use crate::{alloc::flags, bindings, container_of, init, prelude::*, str::CString, types::Opaque};
use core::{
    cell::UnsafeCell,
    ffi::{c_char, c_void},
    marker::PhantomData,
};

const CONFIGFS_PAGE_SIZE: usize = bindings::PAGE_SIZE as usize;

#[pin_data(PinnedDrop)]
pub struct Subsystem<Data> {
    #[pin]
    subsystem: Opaque<bindings::configfs_subsystem>,
    #[pin]
    data: Data,
}

unsafe impl<Data> Sync for Subsystem<Data> {}
unsafe impl<Data> Send for Subsystem<Data> {}

impl<Data> Subsystem<Data> {
    pub fn new(
        name: &'static CStr,
        item_type: &'static ItemType<Subsystem<Data>, Data>,
        data: impl PinInit<Data, Error>,
    ) -> impl PinInit<Self, Error> {
        try_pin_init!(Self {
            subsystem <- init::zeroed().chain(
                |place: &mut Opaque<bindings::configfs_subsystem>| {
                    unsafe {
                        bindings::config_group_init_type_name(
                            &mut (*place.get()).su_group,
                            name.as_char_ptr(),
                            item_type.as_ptr(),
                        )
                    };
                    unsafe {
                        bindings::__mutex_init(
                            &mut (*place.get()).su_mutex,
                            kernel::optional_name!().as_char_ptr(),
                            kernel::static_lock_class!().as_ptr(),
                        )
                    };
                    Ok(())
                }
            ),
            data <- data,
        })
        .pin_chain(|this| unsafe {
            crate::error::to_result(bindings::configfs_register_subsystem(this.subsystem.get()))
        })
    }
}

#[pinned_drop]
impl<Data> PinnedDrop for Subsystem<Data> {
    fn drop(self: Pin<&mut Self>) {
        unsafe { bindings::configfs_unregister_subsystem(self.subsystem.get()) };
    }
}

pub unsafe trait HasGroup<Data> {
    unsafe fn group(this: *const Self) -> *const bindings::config_group;
    unsafe fn container_of(group: *const bindings::config_group) -> *const Self;
}

unsafe impl<Data> HasGroup<Data> for Subsystem<Data> {
    unsafe fn group(this: *const Self) -> *const bindings::config_group {
        unsafe { &raw const (*(*this).subsystem.get()).su_group }
    }

    unsafe fn container_of(group: *const bindings::config_group) -> *const Self {
        let c_subsys_ptr = unsafe { container_of!(group, bindings::configfs_subsystem, su_group) };
        let opaque_ptr = c_subsys_ptr.cast::<Opaque<bindings::configfs_subsystem>>();
        unsafe { container_of!(opaque_ptr, Subsystem<Data>, subsystem) }
    }
}

#[pin_data]
pub struct Group<Data> {
    #[pin]
    group: Opaque<bindings::config_group>,
    #[pin]
    data: Data,
}

impl<Data> Group<Data> {
    pub fn new(
        name: CString,
        item_type: &'static ItemType<Group<Data>, Data>,
        data: impl PinInit<Data, Error>,
    ) -> impl PinInit<Self, Error> {
        try_pin_init!(Self {
            group <- init::zeroed().chain(|place: &mut Opaque<bindings::config_group>| {
                unsafe {
                    bindings::config_group_init_type_name(
                        place.get(),
                        name.as_bytes_with_nul().as_ptr().cast(),
                        item_type.as_ptr(),
                    )
                };
                Ok(())
            }),
            data <- data,
        })
    }

    pub fn data(&self) -> &Data {
        &self.data
    }
}

unsafe impl<Data> HasGroup<Data> for Group<Data> {
    unsafe fn group(this: *const Self) -> *const bindings::config_group {
        Opaque::raw_get(unsafe { &raw const (*this).group })
    }

    unsafe fn container_of(group: *const bindings::config_group) -> *const Self {
        let opaque_ptr = group.cast::<Opaque<bindings::config_group>>();
        unsafe { container_of!(opaque_ptr, Self, group) }
    }
}

unsafe fn get_group_data<'a, Parent>(this: *mut bindings::config_group) -> &'a Parent {
    let is_root = unsafe { (*this).cg_subsys.is_null() };
    if !is_root {
        unsafe { &(*Group::<Parent>::container_of(this)).data }
    } else {
        unsafe { &(*Subsystem::<Parent>::container_of(this)).data }
    }
}

struct GroupOperationsVTable<Parent, Child>(PhantomData<(Parent, Child)>);

impl<Parent, Child> GroupOperationsVTable<Parent, Child>
where
    Parent: GroupOperations<Child = Child>,
    Child: 'static,
{
    unsafe extern "C" fn make_group(
        this: *mut bindings::config_group,
        name: *const c_char,
    ) -> *mut bindings::config_group {
        let parent_data = unsafe { get_group_data(this) };
        let group_init = match Parent::make_group(parent_data, unsafe { CStr::from_char_ptr(name) })
        {
            Ok(init) => init,
            Err(e) => return e.to_ptr(),
        };

        let child_group = Box::pin_init(group_init, flags::GFP_KERNEL);

        match child_group {
            Ok(child_group) => {
                // SAFETY: The child group stays pinned inside the box until the
                // matching release callback reconstructs ownership.
                let child_group_ptr =
                    Box::into_raw(unsafe { Pin::into_inner_unchecked(child_group) });
                unsafe { Group::<Child>::group(child_group_ptr) }.cast_mut()
            }
            Err(e) => e.to_ptr(),
        }
    }

    unsafe extern "C" fn drop_item(
        this: *mut bindings::config_group,
        item: *mut bindings::config_item,
    ) {
        let parent_data = unsafe { get_group_data(this) };
        let c_child_group_ptr = unsafe { container_of!(item, bindings::config_group, cg_item) };
        let r_child_group_ptr = unsafe { Group::<Child>::container_of(c_child_group_ptr) };

        if Parent::HAS_DROP_ITEM {
            let child = unsafe { &*r_child_group_ptr };
            Parent::drop_item(parent_data, child);
        }

        unsafe { bindings::config_item_put(item) };
    }

    const VTABLE: bindings::configfs_group_operations = bindings::configfs_group_operations {
        make_item: None,
        make_group: Some(Self::make_group),
        disconnect_notify: None,
        drop_item: Some(Self::drop_item),
    };

    const fn vtable_ptr() -> *const bindings::configfs_group_operations {
        &Self::VTABLE
    }
}

struct ItemOperationsVTable<Container, Data>(PhantomData<(Container, Data)>);

impl<Data> ItemOperationsVTable<Group<Data>, Data>
where
    Data: 'static,
{
    unsafe extern "C" fn release(this: *mut bindings::config_item) {
        let c_group_ptr = unsafe { container_of!(this, bindings::config_group, cg_item) };
        let r_group_ptr = unsafe { Group::<Data>::container_of(c_group_ptr) };
        // SAFETY: The raw pointer originates from `Box::into_raw` in
        // `make_group`, and this release callback is the unique reclamation
        // point.
        let owned = unsafe { Pin::new_unchecked(Box::from_raw(r_group_ptr.cast_mut())) };
        drop(owned);
    }

    const VTABLE: bindings::configfs_item_operations = bindings::configfs_item_operations {
        release: Some(Self::release),
        allow_link: None,
        drop_link: None,
    };

    const fn vtable_ptr() -> *const bindings::configfs_item_operations {
        &Self::VTABLE
    }
}

impl<Data> ItemOperationsVTable<Subsystem<Data>, Data> {
    const VTABLE: bindings::configfs_item_operations = bindings::configfs_item_operations {
        release: None,
        allow_link: None,
        drop_link: None,
    };

    const fn vtable_ptr() -> *const bindings::configfs_item_operations {
        &Self::VTABLE
    }
}

#[vtable]
pub trait GroupOperations {
    type Child: 'static;

    fn make_group(&self, name: &CStr) -> Result<impl PinInit<Group<Self::Child>, Error>>;

    fn drop_item(&self, _child: &Group<Self::Child>) {
        kernel::build_error!(kernel::error::VTABLE_DEFAULT_ERROR)
    }
}

#[repr(transparent)]
pub struct Attribute<const ID: u64, O, Data> {
    attribute: Opaque<bindings::configfs_attribute>,
    _p: PhantomData<(O, Data)>,
}

unsafe impl<const ID: u64, O, Data> Sync for Attribute<ID, O, Data> {}
unsafe impl<const ID: u64, O, Data> Send for Attribute<ID, O, Data> {}

impl<const ID: u64, O, Data> Attribute<ID, O, Data>
where
    O: AttributeOperations<ID, Data = Data>,
{
    unsafe extern "C" fn show(item: *mut bindings::config_item, page: *mut c_char) -> isize {
        let c_group = unsafe { container_of!(item, bindings::config_group, cg_item) }.cast_mut();
        let data: &Data = unsafe { get_group_data(c_group) };
        let ret = O::show(data, unsafe {
            &mut *(page.cast::<[u8; CONFIGFS_PAGE_SIZE]>())
        });
        match ret {
            Ok(size) => size as isize,
            Err(err) => err.to_errno() as isize,
        }
    }

    unsafe extern "C" fn store(
        item: *mut bindings::config_item,
        page: *const c_char,
        size: usize,
    ) -> isize {
        let c_group = unsafe { container_of!(item, bindings::config_group, cg_item) }.cast_mut();
        let data: &Data = unsafe { get_group_data(c_group) };
        let ret = O::store(data, unsafe {
            core::slice::from_raw_parts(page.cast(), size)
        });
        match ret {
            Ok(()) => size as isize,
            Err(err) => err.to_errno() as isize,
        }
    }

    pub const fn new(name: &'static CStr) -> Self {
        Self {
            attribute: Opaque::new(bindings::configfs_attribute {
                ca_name: name.as_char_ptr(),
                ca_owner: core::ptr::null_mut(),
                ca_mode: 0o660,
                show: Some(Self::show),
                store: if O::HAS_STORE {
                    Some(Self::store)
                } else {
                    None
                },
            }),
            _p: PhantomData,
        }
    }

    pub const fn as_ptr(&self) -> *mut c_void {
        self as *const Self as *mut c_void
    }
}

#[vtable]
pub trait AttributeOperations<const ID: u64 = 0> {
    type Data;

    fn show(data: &Self::Data, page: &mut [u8; CONFIGFS_PAGE_SIZE]) -> Result<usize>;

    fn store(_data: &Self::Data, _page: &[u8]) -> Result {
        kernel::build_error!(kernel::error::VTABLE_DEFAULT_ERROR)
    }
}

#[repr(transparent)]
pub struct AttributeList<const N: usize, Data>(UnsafeCell<[*mut c_void; N]>, PhantomData<Data>);

unsafe impl<const N: usize, Data> Send for AttributeList<N, Data> {}
unsafe impl<const N: usize, Data> Sync for AttributeList<N, Data> {}

impl<const N: usize, Data> AttributeList<N, Data> {
    pub const unsafe fn new() -> Self {
        Self(UnsafeCell::new([core::ptr::null_mut(); N]), PhantomData)
    }

    pub const fn from_raw(attributes: [*mut c_void; N]) -> Self {
        Self(UnsafeCell::new(attributes), PhantomData)
    }

    pub unsafe fn add<const I: usize, const ID: u64, O>(
        &'static self,
        attribute: &'static Attribute<ID, O, Data>,
    ) where
        O: AttributeOperations<ID, Data = Data>,
    {
        const { assert!(I < N - 1, "Invalid attribute index") };
        unsafe {
            (&mut *self.0.get())[I] = (attribute as *const Attribute<ID, O, Data>)
                .cast_mut()
                .cast();
        };
    }
}

#[pin_data]
pub struct ItemType<Container, Data> {
    #[pin]
    item_type: Opaque<bindings::config_item_type>,
    _p: PhantomData<(Container, Data)>,
}

unsafe impl<Container, Data> Sync for ItemType<Container, Data> {}
unsafe impl<Container, Data> Send for ItemType<Container, Data> {}

macro_rules! impl_item_type {
    ($tpe:ty) => {
        impl<Data> ItemType<$tpe, Data> {
            pub const fn new_with_child_ctor<const N: usize, Child>(
                attributes: &'static AttributeList<N, Data>,
            ) -> Self
            where
                Data: GroupOperations<Child = Child>,
                Child: 'static,
            {
                Self {
                    item_type: Opaque::new(bindings::config_item_type {
                        ct_owner: core::ptr::null_mut(),
                        ct_group_ops: GroupOperationsVTable::<Data, Child>::vtable_ptr().cast_mut(),
                        ct_item_ops: ItemOperationsVTable::<$tpe, Data>::vtable_ptr().cast_mut(),
                        ct_attrs: (attributes as *const AttributeList<N, Data>)
                            .cast_mut()
                            .cast(),
                        ct_bin_attrs: core::ptr::null_mut(),
                    }),
                    _p: PhantomData,
                }
            }

            pub const fn new<const N: usize>(attributes: &'static AttributeList<N, Data>) -> Self {
                Self {
                    item_type: Opaque::new(bindings::config_item_type {
                        ct_owner: core::ptr::null_mut(),
                        ct_group_ops: core::ptr::null_mut(),
                        ct_item_ops: ItemOperationsVTable::<$tpe, Data>::vtable_ptr().cast_mut(),
                        ct_attrs: (attributes as *const AttributeList<N, Data>)
                            .cast_mut()
                            .cast(),
                        ct_bin_attrs: core::ptr::null_mut(),
                    }),
                    _p: PhantomData,
                }
            }
        }
    };
}

impl_item_type!(Subsystem<Data>);
impl_item_type!(Group<Data>);

impl<Container, Data> ItemType<Container, Data> {
    fn as_ptr(&self) -> *const bindings::config_item_type {
        self.item_type.get()
    }
}

#[macro_export]
macro_rules! configfs_attrs {
    (
        container: $container:ty,
        data: $data:ty,
        attributes: [
            $($name:ident: $attr:literal),* $(,)?
        ] $(,)?
    ) => {
        $crate::configfs_attrs!(
            count:
            @container($container),
            @data($data),
            @child(),
            @no_child(x),
            @eat($($name $attr,)*),
            @assign(),
            @cnt(0usize),
        )
    };
    (
        container: $container:ty,
        data: $data:ty,
        child: $child:ty,
        attributes: [
            $($name:ident: $attr:literal),* $(,)?
        ] $(,)?
    ) => {
        $crate::configfs_attrs!(
            count:
            @container($container),
            @data($data),
            @child($child),
            @no_child(),
            @eat($($name $attr,)*),
            @assign(),
            @cnt(0usize),
        )
    };
    (count:
     @container($container:ty),
     @data($data:ty),
     @child($($child:ty)?),
     @no_child($($no_child:ident)?),
     @eat($name:ident $attr:literal, $($rname:ident $rattr:literal,)*),
     @assign($($assign:block)*),
     @cnt($cnt:expr),
    ) => {
        $crate::configfs_attrs!(
            count:
            @container($container),
            @data($data),
            @child($($child)?),
            @no_child($($no_child)?),
            @eat($($rname $rattr,)*),
            @assign($($assign)* {
                const N: usize = $cnt;
                unsafe {
                    $crate::macros::paste!(
                        [< $data:upper _ATTRS >]
                            .add::<N, $attr, _>(&[< $data:upper _ $name:upper _ATTR >])
                    )
                };
            }),
            @cnt(1usize + $cnt),
        )
    };
    (count:
     @container($container:ty),
     @data($data:ty),
     @child($($child:ty)?),
     @no_child($($no_child:ident)?),
     @eat(),
     @assign($($assign:block)*),
     @cnt($cnt:expr),
    ) => {
        $crate::configfs_attrs!(
            final:
            @container($container),
            @data($data),
            @child($($child)?),
            @no_child($($no_child)?),
            @assign($($assign)*),
            @cnt($cnt),
        )
    };
    (final:
     @container($container:ty),
     @data($data:ty),
     @child($($child:ty)?),
     @no_child($($no_child:ident)?),
     @assign($($assign:block)*),
     @cnt($cnt:expr),
    ) => {
        $crate::macros::paste! {{
            static [< $data:upper _ATTRS >]:
                $crate::configfs::AttributeList<{ $cnt + 1usize }, $data> =
                    unsafe { $crate::configfs::AttributeList::new() };

            $($assign)*

            $(
                const [<$no_child:upper>]: bool = true;
                static [< $data:upper _TPE >]: $crate::configfs::ItemType<$container, $data> =
                    $crate::configfs::ItemType::<$container, $data>::new::<{ $cnt + 1usize }>(
                        &[< $data:upper _ATTRS >]
                    );
            )?

            $(
                static [< $data:upper _TPE >]: $crate::configfs::ItemType<$container, $data> =
                    $crate::configfs::ItemType::<$container, $data>::new_with_child_ctor::<
                        { $cnt + 1usize },
                        $child
                    >(&[< $data:upper _ATTRS >]);
            )?

            & [< $data:upper _TPE >]
        }}
    };
}

pub use crate::configfs_attrs;
