.. SPDX-License-Identifier: GPL-2.0

2026-03-17 工作日志：Rust-for-Linux 开发基线搭建
================================================

日期
----

- 日期：2026-03-17
- 记录人：``lwz23``
- 主题：建立可长期维护的 Rust-for-Linux 本地开发环境，并完成首次 GitHub 正式化。

今日目标
--------

今天的目标不是修一个单点问题，而是把 Rust-for-Linux 的开发方式从
“旧环境能否偶然跑起来”升级成“可持续维护、可重复构建、可持续推送”的正式项目。

具体目标包括：

- 不再把旧目录 ``/home/lwz/linux`` 作为长期开发基线。
- 在 ``/home/lwz/rfl-dev`` 下建立新的工作区。
- 使用 ``Rust-for-Linux/linux`` 的 ``rust-next`` 作为上游基线。
- 使用新的 BusyBox rootfs 和 QEMU 运行真实 Rust ``.ko`` 模块。
- 建立 GitHub fork、``origin/dev`` 分支以及后续长期维护所需的本地规范。

今日完成事项
------------

1. 新工作区与目录布局
~~~~~~~~~~~~~~~~~~~~~

在宿主机上建立了新的长期工作区 ``/home/lwz/rfl-dev``，并固定以下结构：

- ``linux``: 内核源码仓库。
- ``build``: 分离输出目录。
- ``toolchains``: 独立 LLVM + Rust 工具链。
- ``rootfs``: 新 BusyBox initramfs。
- ``modules/rust-out-of-tree-module``: 官方模块模板。

旧目录 ``/home/lwz/linux`` 保留为历史参考，没有被 reset、覆盖或纳入新闭环。

2. 仓库与分支初始化
~~~~~~~~~~~~~~~~~~~

完成了本地 Git 组织方式的重建：

- ``upstream`` 指向官方 ``https://github.com/Rust-for-Linux/linux.git``。
- ``origin`` 最终指向 GitHub fork ``git@github.com:lwz23/rust-for-linux-dev.git``。
- 本地从 ``upstream/rust-next`` 建立 ``dev`` 分支。
- 最终把 ``dev`` 成功推送为 ``origin/dev``。

在推送远端时还确认了一个关键事实：把 shallow clone 直接推到空 GitHub 仓库
并不稳定，远端会在 unpack 阶段失败。最终改为先创建官方仓库的 fork，再把
本地 ``dev`` 推到 fork 上，问题解决。

3. 独立工具链与 Rust 构建环境
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

搭建了独立于系统默认环境的新工具链，并固定在：

- ``/home/lwz/rfl-dev/toolchains/llvm-22.1.0-rust-1.93.1-x86_64``

实际工作版本如下：

- ``clang 22.1.0``
- ``rustc 1.93.1``
- ``bindgen 0.72.1``

最初计划希望使用 ``bindgen 0.65.1``，但在这台机器和这套工具链组合下，
低版本 bindgen 生成的若干内核 Rust bindings 出现异常，导致关键结构体字段
缺失。最终改用 ``bindgen 0.72.1`` 后问题消失，构建恢复正常。

4. 新内核构建与配置
~~~~~~~~~~~~~~~~~~~

完成了新内核树的独立配置和构建：

- 配置起点：``x86_64_defconfig``
- 显式开启：``CONFIG_RUST=y``、``CONFIG_MODULES=y``、``CONFIG_MODULE_UNLOAD=y``
- 显式关闭：``CONFIG_DEBUG_INFO_BTF``
- ``rustavailable`` 检查通过
- 成功生成 ``bzImage``

内核构建输出目录固定为 ``/home/lwz/rfl-dev/build``，后续不污染源码树。

5. 新 rootfs 与真实 Rust 模块加载验证
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

没有复用旧 ``initrd.img``，而是新建了最小 BusyBox initramfs。
``/init`` 会在启动后自动：

- 挂载 ``proc``、``sysfs``、``devtmpfs``
- 执行 ``mdev -s``
- 自动 ``insmod /lib/modules/rust_out_of_tree.ko``
- 打印 ``rust_out_of_tree`` 相关日志
- 进入交互 shell

随后完成了真实 out-of-tree Rust 模块的端到端验证：

- 模块文件：``/home/lwz/rfl-dev/modules/rust-out-of-tree-module/rust_out_of_tree.ko``
- QEMU 成功启动新内核
- 启动阶段自动加载模块成功
- 进入 guest shell 后执行 ``rmmod rust_out_of_tree`` 成功

关键日志如下：

.. code-block:: text

   [    0.731582] rust_out_of_tree: loading out-of-tree module taints kernel.
   [    0.731900] rust_out_of_tree: Rust out-of-tree sample (init)
   [    8.060218] rust_out_of_tree: My numbers are [72, 108, 200]
   [    8.060481] rust_out_of_tree: Rust out-of-tree sample (exit)

这说明今天已经完成了“真实 Rust 模块被新内核成功加载和卸载”的闭环。

6. 宿主机便捷脚本
~~~~~~~~~~~~~~~~~

为了减少重复手工操作，在 ``/home/lwz/rfl-dev/scripts`` 下整理了便捷脚本：

- ``configure-kernel.sh``
- ``build-kernel.sh``
- ``prepare-rootfs.sh``
- ``build-module.sh``
- ``build-initramfs.sh``
- ``run-qemu.sh``
- ``sync-upstream.sh``
- ``publish-dev.sh``

这些脚本属于宿主机工作区，不在当前 Git 仓库内，因此今天又补写了
``Documentation/rust/lwz-dev/development-guide-zh_CN.rst``，把对应原生命令也记录下来，
避免未来只能依赖脚本文件本身。

7. GitHub 认证与正式项目化
~~~~~~~~~~~~~~~~~~~~~~~~~~

今天还完成了 GitHub 认证链路的打通：

- 确认本机已有可用 SSH key，并且 GitHub 账号 ``lwz23`` 已绑定。
- 配置 ``~/.ssh/config``，强制 GitHub SSH 走 443 端口，以适应当前网络环境。
- 验证 ``ssh -T git@github.com`` 可成功认证。
- 通过 fork 方式建立正式 GitHub 仓库 ``lwz23/rust-for-linux-dev``。
- 成功推送 ``origin/dev`` 并让本地 ``dev`` 跟踪 ``origin/dev``。

当前正式开发基线已经具备：

- 本地可构建。
- 本地可启动。
- 模块可真实验证。
- GitHub 有正式远端。
- ``dev`` 可作为长期主开发分支。

8. in-tree Rust sample 整内核重编验证
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

在完成 out-of-tree Rust 模块验证之后，今天还进一步验证了
“修改内核树内部的 Rust 代码并重新编译整个内核”这条链路。

本次选择的目标是内核树中的 ``samples/rust/rust_minimal.rs``。
这样做的原因是：

- 它是官方 in-tree Rust sample，适合做低风险验证。
- 逻辑简单，只需要改少量 ``pr_info!`` 日志即可。
- 可以通过 built-in 方式直接在启动日志里确认是否生效。

本次操作包括：

- 修改 ``samples/rust/rust_minimal.rs`` 中的启动和退出日志，加入 ``lwz`` 标记。
- 在构建配置中开启 ``CONFIG_SAMPLES=y``。
- 在构建配置中开启 ``CONFIG_SAMPLES_RUST=y``。
- 在构建配置中开启 ``CONFIG_SAMPLE_RUST_MINIMAL=y``。
- 重新编译整个内核并重新启动 QEMU。

最终在 QEMU 启动日志中观察到了下面这组关键输出：

.. code-block:: text

   [    0.672622] rust_minimal: lwz: Rust minimal sample (init)
   [    0.672920] rust_minimal: Am I built-in? true
   [    0.673160] rust_minimal: test_parameter: 1

这组日志说明：

- ``rust_minimal`` 已经成功编译进内核映像，而不是以独立模块形式加载。
- 修改过的 Rust 源码已经进入新 ``bzImage``，并在启动早期执行。
- 当前环境不仅支持 out-of-tree Rust 模块验证，也支持 in-tree Rust 代码的整内核重编验证。

今日产出
--------

今天的重要产出包括：

- 新工作区 ``/home/lwz/rfl-dev``
- 可启动的新内核 ``bzImage``
- 可加载的 Rust out-of-tree 模块 ``rust_out_of_tree.ko``
- 成功验证的 in-tree Rust sample ``rust_minimal``
- 新 BusyBox initramfs
- GitHub fork ``lwz23/rust-for-linux-dev``
- 本地 ``dev`` 与远端 ``origin/dev`` 的长期开发关系
- 本目录下的中文开发说明与工作日志

关键结论
--------

今天最重要的结论有五点：

1. 旧环境不再适合作为长期开发主线，新环境已经成功替代它。
2. 当前 Rust-for-Linux 项目在这台机器上是可以正常跑起来的，而且不是“只编过”，
   而是已经完成了真实模块加载与卸载验证。
3. 当前环境不仅支持 out-of-tree Rust 模块，也支持 in-tree Rust 代码改动后的整内核重编验证。
4. 把浅克隆直接推到空仓库不可靠，长期维护应以 GitHub fork 为 ``origin``。
5. 文档、分支和构建命令如果不同时规范化，后续维护成本会很快上升，因此今天优先把
   开发说明和工作记录一起补齐是必要动作。

遗留事项与下一步建议
--------------------

今天之后，建议按下面的顺序继续推进：

1. 在 GitHub 网页中把仓库默认分支从 ``rust-next`` 调整为 ``dev``。
2. 后续所有实际开发都从 ``dev`` 切 ``feature/<topic>``。
3. 每轮开发结束前都至少完成一次模块加载和卸载验证。
4. 如果后续要做整内核 Rust 改动，可把 ``rust_minimal`` 作为 smoke test，用来快速确认新的 ``bzImage`` 确实生效。
5. 需要同步上游时，先合并 ``upstream/rust-next`` 到 ``dev``，再开始新工作。
6. 如果后续需要更完整的历史，再考虑对本地内核仓库执行 ``git fetch --unshallow upstream``。

备注
----

这份日志的用途不是替代 commit message，而是记录“今天做了哪些基础设施工作、
为什么这样做、有哪些坑、最后状态如何”。以后如果继续进行环境演进，建议继续在
本目录追加新的工作日志文件，保持长期可追溯性。
