# Move Line Rules

### Rule 1: parameter validation can be placed in the first line
Parameter validation with assertions can be placed at the beginning of functions. If failed, gas can be saved.
- source code: [detector1.rs](./detector1.rs)
- test case: [Detector1](../../../tests/cases/Detector1)

### Rule 2: assert error code usage
If assert error code is undefined, use 0 directly.
- source code: [detector2.rs](./detector2.rs)
- test case: [Detector2](../../../tests/cases/Detector2)

### Rule 3: unnecessary type conversion
Unnecessary type conversion, for example let a: u64; a as u64;
- source code: [detector3.rs](./detector3.rs)
- test case: [Detector3](../../../tests/cases/Detector3)

### Rule 4: unused private interface
Unused private interface exists.
- source code: [detector4.rs](./detector4.rs)
- test case: [Detector4](../../../tests/cases/Detector4)

### Rule 5: shift operation overflow
Make sure that the second operand is less than the width in bits of the first operand and no overflow during a shift operation.
- source code: [detector5.rs](./detector5.rs)
- test case: [Detector5](../../../tests/cases/Detector5)

### Rule 6: call deprecated functions of other modules
Call deprecated functions of other modules which may lead to logic errors.
- source code: [detector6.rs](./detector6.rs)
- test case: [Detector6](../../../tests/cases/Detector6)
- TODO: the set of deprecated functions to be completed

### Rule 7: multiplication comes before division
Multiplication comes before division, otherwise the result precision may be lower.
- source code: [detector7.rs](./detector7.rs)
- test case: [Detector7](../../../tests/cases/Detector8)

### Rule 8: inexplicit version of dependent libraries
The version of dependent libraries should be a version number or commit number and avoid to use branch names.
- source code: [detector8.rs](./detector8.rs)
- test case: [Detector8](../../../tests/cases/Detector8)
