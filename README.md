# Move Lint
Move语言的静态检测工具。工具只需要用户传入智能合约，就能自动化的发现合约中的潜在安全隐患，定位漏洞产生的位置，增强合约的安全性。

工具主要包含两大方面的检测：
1. 代码规范性检测。此项针对合约编写时的一些代码规范进行检测，不规范的代码编写引发安全问题的可能性会大大提高。
2. 常规安全问题检测。此项针对合约中常见的安全问题进行检测。常规安全问题是所有合约都可能出现的安全问题，与业务逻辑无关。

工具自带检测[规则库](src/lint/detectors/README.md)，提供了方便的增删接口，可以随着Move合约业务的发展，快速适配新的检测规则，提高工具的检测能力。

# Development

### Building
```bash
git clone xx/xx/move-line.git
cd move-lint
cargo build --release
```

### Help
```bash
cd target/release
./move-lint -h
```
```
USAGE:
    move-lint [OPTIONS]

OPTIONS:
    -h, --help                   Print help information
    -j, --json                   Print results as json if available
    -p, --path <PACKAGE_PATH   Path to a package which the command should be run with respect to
    -v                           Print additional diagnostics if available
    -V, --version                Print version information
```

### Example
```bash
./move-lint -p ../../tests/cases/Detector1
```
```
no: 1
wiki: 
title: 参数校验可以放在首行
verbose: 参数校验的assert可放在函数开头，快速失败，省gas
level: Warning
description: None
file: ./sources/Detector.move
range: (128, 129)
lines: [5]


no: 1
wiki: 
title: 参数校验可以放在首行
verbose: 参数校验的assert可放在函数开头，快速失败，省gas
level: Warning
description: None
file: ./sources/Detector.move
range: (237, 238)
lines: [9]


no: 1
wiki: 
title: 参数校验可以放在首行
verbose: 参数校验的assert可放在函数开头，快速失败，省gas
level: Warning
description: None
file: ./sources/Detector.move
range: (349, 350)
lines: [13]
```

