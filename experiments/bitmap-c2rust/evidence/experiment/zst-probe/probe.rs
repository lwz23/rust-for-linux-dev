// 最小复现：与 RFL Bitmap 相同的 ZST 切片 DST 包装，指针来自 &Bitmap
#[repr(align(8))]
pub struct Bitmap { data: [()] }
impl Bitmap {
    pub unsafe fn from_raw<'a>(ptr: *const usize, nbits: usize) -> &'a Bitmap {
        let data: *const [()] = core::ptr::slice_from_raw_parts(ptr.cast(), nbits);
        unsafe { &*(data as *const Bitmap) }
    }
    pub fn as_ptr(&self) -> *const usize { core::ptr::from_ref::<Bitmap>(self).cast::<usize>() }
}
fn main() {
    let words: [usize; 2] = [0, 1];
    let b = unsafe { Bitmap::from_raw(words.as_ptr(), 128) };
    let v = unsafe { *b.as_ptr().add(1) }; // 纯 Rust 读取，没有任何 C / C2Rust
    println!("{v}");
}
