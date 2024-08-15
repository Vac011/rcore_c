# Proj_146
支持优先级的rust协程调度
* 学校：武汉大学
* 队名：rust协程不排队
* 成员：葛洋、张珈豪、李晋
* 指导教师：龚奕利、胡创

本项目结构：
```
rCore_c
├─ code
├─ docs
├─ readme.md
└─ test
```
* code包含rCore-c的代码文件
* docs为本项目的文档
* test为测试文件

演示视频链接：https://pan.baidu.com/s/17vZi1Esu1OBhspXPU4ABpQ 

提取码：cbr0


技术文档链接：[rCore-c.md](./docs/rCore-c.md)

代码链接：[rCore-c](./code)

运行：

配置好rust环境后，进入./code/os目录，在终端中运行> make run 命令，即可在qemu中运行rCore-c

根据shell提示输入测试程序名进行测试

相关项目：

[rCore](https://github.com/rcore-os/rCore-Tutorial-v3)
[tornado-os](https://github.com/HUST-OS/tornado-os)
[rCore-N](https://github.com/duskmoon314/rCore-N)