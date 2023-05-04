# Move Lint
A static detection tool for Move language. The tool takes a smart contract as input and can automatically discover potential security vulnerabilities in the contract, locate the codes which generate the vulnerabilities, and enhance the security of the contract.   

The tool mainly includes two types of detection:
1. Code conventions detection. It detects any violations of certain code conventions during the development of a smart contract. The possibility of introducing security vulnerabilities is greatly increased by inappropriate code writing.
2. Regular security issues detection. It detects regular security issues in the contracts. Regular security issues are issues that may exist in any contract and are independent of business logic, e.g. division before multiplication.

The tool has a [detection rule library](src/lint/detectors/README.md), providing convenient addition and deletion interfaces. It can quickly adapt to new detection rules and improve its detection abilities as the business development of the Move smart contracts.

# Development

### Building
```bash
git clone https://github.com/BeosinBlockchainSecurity/Move-Lint.git
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
    -p, --path <PACKAGE_PATH>    Path to a package which the command should be run with respect to
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
title: parameter validation can be placed in the first line
verbose: Parameter validation with assertions can be placed at the beginning of functions. If failed, gas can be saved.
level: Warning
description: None
file: ./sources/Detector.move
range: (128, 129)
lines: [5]


no: 1
wiki: 
title: parameter validation can be placed in the first line
verbose: Parameter validation with assertions can be placed at the beginning of functions. If failed, gas can be saved.
level: Warning
description: None
file: ./sources/Detector.move
range: (237, 238)
lines: [9]


no: 1
wiki: 
title: parameter validation can be placed in the first line
verbose: Parameter validation with assertions can be placed at the beginning of functions. If failed, gas can be saved.
level: Warning
description: None
file: ./sources/Detector.move
range: (349, 350)
lines: [13]
```

