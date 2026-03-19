.. SPDX-License-Identifier: GPL-2.0

Rust-for-Linux 本地开发说明
===========================

适用范围
--------

本文档描述 ``lwz23`` 当前在宿主机上使用的 Rust-for-Linux 长期开发方式。
它不是通用上游说明，而是对当前工作区、分支策略、提交习惯和常用命令的
本地约定，目的是让后续开发、验证和回顾都可重复。

工作区固定布局
--------------

当前工作区固定放在 ``/home/lwz/rfl-dev``，各目录职责如下：

- ``/home/lwz/rfl-dev/linux``: Rust-for-Linux 内核源码仓库，当前主开发分支为 ``dev``。
- ``/home/lwz/rfl-dev/build``: 内核分离输出目录，放置 ``.config``、``vmlinux``、
  ``bzImage`` 和模块构建所需 metadata。
- ``/home/lwz/rfl-dev/toolchains``: 独立 LLVM + Rust 工具链。
- ``/home/lwz/rfl-dev/rootfs``: BusyBox initramfs 的 staging 目录和打包产物。
- ``/home/lwz/rfl-dev/modules/rust-out-of-tree-module``: 官方 out-of-tree Rust 模块模板。
- ``/home/lwz/rfl-dev/scripts``: 宿主机上使用的便捷脚本，不属于当前 Git 仓库，
  但本文档会记录对应命令。

当前固定环境变量来自 ``/home/lwz/rfl-dev/env.sh``，核心变量如下：

- ``RFL_DEV_ROOT=/home/lwz/rfl-dev``
- ``KERNEL_SRC=/home/lwz/rfl-dev/linux``
- ``KERNEL_BUILD=/home/lwz/rfl-dev/build``
- ``MODULE_SRC=/home/lwz/rfl-dev/modules/rust-out-of-tree-module``
- ``ROOTFS_DIR=/home/lwz/rfl-dev/rootfs/stage``
- ``ROOTFS_IMAGE=/home/lwz/rfl-dev/rootfs/initramfs.cpio.gz``

仓库与远程规范
--------------

当前仓库使用两个 remote：

- ``upstream``: ``https://github.com/Rust-for-Linux/linux.git``
- ``origin``: ``git@github.com:lwz23/rust-for-linux-dev.git``

其中 ``origin`` 应保持为 GitHub fork，而不是空仓库。这样远端天然带有
``rust-next`` 历史，本地 ``dev`` 只需要在其上追加用户自己的开发提交。

当前 GitHub 仓库的建议状态如下：

- ``origin/rust-next``: 对应 fork 同步来的官方分支。
- ``origin/dev``: 本地长期开发集成分支。
- GitHub 默认分支建议手动设置为 ``dev``。

分支规范
--------

本地开发统一使用下面的分支模型：

- ``dev``: 长期开发主线。所有可保留的改动最终都应回到 ``dev``。
- ``feature/<topic>``: 功能开发分支。新功能、重构、实验性验证都从 ``dev`` 切出。

推荐分支流程：

1. 先在 ``dev`` 上同步 ``upstream/rust-next``。
2. 从最新 ``dev`` 切出 ``feature/<topic>``。
3. 在 ``feature/<topic>`` 上完成开发、构建和验证。
4. 清理提交历史后合回 ``dev``。
5. 再把 ``dev`` 推送到 ``origin/dev``。

额外约定：

- 不直接在 ``rust-next`` 上开发。
- 不直接在 ``origin/rust-next`` 上提交用户改动。
- ``dev`` 尽量保持为“随时可构建、可启动、可验证模块”的状态。
- 只有在明确知道影响范围时才对共享分支做 history rewrite。

提交规范
--------

整体目标是保持提交历史“像内核提交”，同时方便后续回看和必要时上游化。

推荐规则如下：

- 一个 commit 只做一件逻辑上完整的事情。
- 标题使用内核常见格式：``<subsystem>: <summary>``。
- 标题尽量控制在一行内，避免冗长描述。
- 正文优先解释“为什么改”和“影响了什么”，而不是重复 diff。
- 如果提交未来可能上游，优先使用 ``git commit -s`` 带上 ``Signed-off-by``。
- 合并到 ``dev`` 前，尽量整理掉无意义的 ``fix typo``、``wip``、``try again`` 之类噪声提交。

推荐标题示例：

- ``rust: add helper for xxx``
- ``samples: rust: update out-of-tree test flow``
- ``Documentation: rust: record local dev workflow``
- ``Documentation: rust: add work log for 2026-03-17 bootstrap``

推荐提交模板：

.. code-block:: text

   Documentation: rust: add local development guide

   Record the local Rust-for-Linux workflow used in /home/lwz/rfl-dev,
   including remotes, branching rules, build commands, and QEMU validation
   steps so that the environment can be rebuilt and maintained over time.

   Signed-off-by: lwz23 <wenzhaoliao@ruc.edu.cn>

如果某个功能分支已经推到远端并可能被其他人基于其继续工作，除非必要，
否则不要对该分支执行强制推送。

模块 Rust 化阶段纪律
--------------------

从 ``nlmon`` 与 ``rnull`` 两轮实验开始，本地开发约定新增下面几条流程纪律：

- 真实工具流程统一称为 ``reference-free production pipeline``。
- 若为了校准流程而选择一个已有官方 Rust 版本的驱动做 benchmark，则属于
  ``reference-based research calibration pipeline``。

1. 新模块 Rust 化固定分成两段
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- ``blind-first bootstrap``：

  - 先在目标旧树上做出一个可构建、可装载、驱动层零 ``unsafe`` 的原型

- ``mainline-grade conformance hardening``：

  - 这是未来真实工具流程中的标准第二阶段
  - 目标是按主线规则收紧对象模型和 API 边界，而不仅仅是跑出相同功能

2. 研究校准是额外分支，不是生产流程硬步骤
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- 如果当前任务是研究型 benchmark，可以额外采用：

  1. 原始 C 实现
  2. 官方初始 upstream Rust 实现
  3. 如有需要，再看当前最新主线成熟版

- 这条比较链只服务于规则校准，不属于未来真实工具的标准运行步骤。

3. 抽象设计默认约束
~~~~~~~~~~~~~~~~~~~

- 对 intrusive / registered / callback-owned C 对象，默认优先：

  - ``Pin + Opaque``
  - 显式状态机
  - builder 前置校验

- ``unsafe impl Send/Sync`` 默认禁止，除非有结构性证明。
- helper 只能在主 bindings 缺口审计确认后生成。
- hardening 阶段必须做 helper 退役审计，防止过渡桥接长期残留。

4. 文档和验收口径纪律
~~~~~~~~~~~~~~~~~~~~~

- ``blind-first`` 完成后，只能写“原型已跑通”。
- 只有 hardening、``unsafe`` 审计刷新和关键验证都完成后，才允许写工程验收结论。
- 如果当前任务是研究型 benchmark，再在 hardening 之后追加 ``reference-based evaluation``。
- 未完成定性的工具链/``objtool`` warning 必须单列为后续事项，不能被并入“测试通过”。

常用命令
--------

环境初始化
~~~~~~~~~~

先加载固定工具链和路径：

.. code-block:: bash

   source /home/lwz/rfl-dev/env.sh

核对工具链：

.. code-block:: bash

   "$RFL_LLVM_PREFIX/bin/clang" --version | head -n 1
   "$RFL_LLVM_PREFIX/bin/rustc" --version
   "$BINDGEN" --version

同步上游到 ``dev``
~~~~~~~~~~~~~~~~~~~

宿主机便捷脚本：

.. code-block:: bash

   /home/lwz/rfl-dev/scripts/sync-upstream.sh

等价原生命令：

.. code-block:: bash

   source /home/lwz/rfl-dev/env.sh
   git -C "$KERNEL_SRC" fetch upstream
   git -C "$KERNEL_SRC" checkout dev
   git -C "$KERNEL_SRC" merge upstream/rust-next
   git -C "$KERNEL_SRC" push origin dev

创建功能分支
~~~~~~~~~~~~

.. code-block:: bash

   source /home/lwz/rfl-dev/env.sh
   git -C "$KERNEL_SRC" checkout dev
   git -C "$KERNEL_SRC" pull --ff-only origin dev
   git -C "$KERNEL_SRC" checkout -b feature/<topic>

查看工作区状态
~~~~~~~~~~~~~~

.. code-block:: bash

   source /home/lwz/rfl-dev/env.sh
   git -C "$KERNEL_SRC" status --short --branch
   git -C "$KERNEL_SRC" branch -vv
   git -C "$KERNEL_SRC" log --oneline --decorate -n 10

配置内核基线
~~~~~~~~~~~~

宿主机便捷脚本：

.. code-block:: bash

   /home/lwz/rfl-dev/scripts/configure-kernel.sh

等价原生命令：

.. code-block:: bash

   source /home/lwz/rfl-dev/env.sh
   make -C "$KERNEL_SRC" O="$KERNEL_BUILD" LLVM=1 x86_64_defconfig
   "$KERNEL_SRC/scripts/config" --file "$KERNEL_BUILD/.config" \
       -e RUST \
       -e MODULES \
       -e MODULE_UNLOAD \
       -d DEBUG_INFO_BTF
   make -C "$KERNEL_SRC" O="$KERNEL_BUILD" LLVM=1 olddefconfig

检查 Rust 构建可用性
~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   source /home/lwz/rfl-dev/env.sh
   make -C "$KERNEL_SRC" O="$KERNEL_BUILD" LLVM=1 rustavailable

构建内核
~~~~~~~~

宿主机便捷脚本：

.. code-block:: bash

   /home/lwz/rfl-dev/scripts/build-kernel.sh

等价原生命令：

.. code-block:: bash

   source /home/lwz/rfl-dev/env.sh
   make -C "$KERNEL_SRC" O="$KERNEL_BUILD" LLVM=1 rustavailable
   make -C "$KERNEL_SRC" O="$KERNEL_BUILD" LLVM=1 -j"$(nproc)" bzImage modules

准备 rootfs
~~~~~~~~~~~

宿主机便捷脚本：

.. code-block:: bash

   /home/lwz/rfl-dev/scripts/prepare-rootfs.sh

说明：

- rootfs 使用新的 BusyBox initramfs，不复用旧 ``/home/lwz/linux/initrd.img``。
- ``/init`` 会自动挂载 ``proc``、``sysfs``、``devtmpfs``，然后加载
  ``/lib/modules/rust_out_of_tree.ko``。

构建 out-of-tree Rust 模块并重打包 initramfs
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

宿主机便捷脚本：

.. code-block:: bash

   /home/lwz/rfl-dev/scripts/build-module.sh

等价原生命令：

.. code-block:: bash

   source /home/lwz/rfl-dev/env.sh
   make -C "$KERNEL_SRC" O="$KERNEL_BUILD" LLVM=1 M="$MODULE_SRC"
   install -D -m 0644 \
       "$MODULE_SRC/rust_out_of_tree.ko" \
       "$ROOTFS_DIR/lib/modules/rust_out_of_tree.ko"
   (
       cd "$ROOTFS_DIR"
       find . -print0 | cpio --null -ov --format=newc 2>/dev/null | gzip -9 > "$ROOTFS_IMAGE"
   )

启动 QEMU 验证
~~~~~~~~~~~~~~

宿主机便捷脚本：

.. code-block:: bash

   /home/lwz/rfl-dev/scripts/run-qemu.sh

等价原生命令：

.. code-block:: bash

   qemu-system-x86_64 \
       -enable-kvm \
       -nographic \
       -m 2048 \
       -kernel /home/lwz/rfl-dev/build/arch/x86/boot/bzImage \
       -initrd /home/lwz/rfl-dev/rootfs/initramfs.cpio.gz \
       -append 'console=ttyS0 rdinit=/init printk.devkmsg=on loglevel=7' \
       -no-reboot

如果宿主机当前无法访问 ``/dev/kvm``，只移除 ``-enable-kvm``，其他参数保持不变。

QEMU 内部最常用的检查命令：

.. code-block:: sh

   dmesg | grep rust_out_of_tree
   rmmod rust_out_of_tree
   dmesg | grep rust_out_of_tree
   poweroff -f

推送功能分支
~~~~~~~~~~~~

.. code-block:: bash

   source /home/lwz/rfl-dev/env.sh
   git -C "$KERNEL_SRC" push -u origin feature/<topic>

将功能分支合回 ``dev``
~~~~~~~~~~~~~~~~~~~~~~

一种简单做法如下：

.. code-block:: bash

   source /home/lwz/rfl-dev/env.sh
   git -C "$KERNEL_SRC" checkout dev
   git -C "$KERNEL_SRC" pull --ff-only origin dev
   git -C "$KERNEL_SRC" merge --no-ff feature/<topic>
   git -C "$KERNEL_SRC" push origin dev

如果功能分支只是一个干净的单提交，也可以在自查后使用 ``--ff-only`` 风格合并。

建议的日常工作节奏
------------------

推荐把每轮开发固定为下面的节奏：

1. ``dev`` 同步 ``upstream/rust-next``。
2. 从 ``dev`` 切出 ``feature/<topic>``。
3. 修改代码并做小步提交。
4. 重新构建内核或模块。
5. 启动 QEMU，验证模块加载和卸载日志。
6. 整理 commit message。
7. 推送分支到 GitHub。
8. 合回 ``dev``，保持 ``dev`` 可运行。

维护提醒
--------

- 这套环境当前依赖宿主机上的绝对路径 ``/home/lwz/rfl-dev``。
- 本地内核仓库目前仍是 shallow clone；如果以后需要完整历史，可再做
  ``git fetch --unshallow upstream``。
- 本地实际工作的 helper scripts 位于 ``/home/lwz/rfl-dev/scripts``，
  它们不在当前 Git 仓库中，因此这里同步保留了原生命令。
- 如果未来要向上游提交补丁，建议进一步遵循上游 ``MAINTAINERS``、
  ``Documentation/process`` 和 Rust 文档中的额外要求。
