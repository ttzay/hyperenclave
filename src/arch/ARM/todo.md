TODO
[ ] 1.context.rs 中实现linuxContext上下文切换
[ ] 2.mem_encrypt.rs 实现ARM的硬件加密方法和软件加密方法，参考AMD mem_encrypt.rs
[ ] 3.npt.rs 根据ARM stage-2 translation的硬件细节，实现类似与npt.rs的方法

华为的arm授权为 v8.2，EL2 only existed in Non-secure state because there was no  virtualization support in Secure state