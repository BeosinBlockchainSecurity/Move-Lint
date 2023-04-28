# Move Lint Rules

### 规则1：参数校验可以放在首行
参数校验的assert可放在函数开头，快速失败，省gas
- 源码：[detector1.rs](./detector1.rs)
- 测试用例：[Detector1](../../../tests/cases/Detector1)

### 规则2：assert错误码使用
对于assert的错误码未定义，直接使用0
- 源码：[detector2.rs](./detector2.rs)
- 测试用例：[Detector2](../../../tests/cases/Detector2)

### 规则3：不必要的类型转换
不必要的类型转换，例如：let a: u64; a as u64;
- 源码：[detector3.rs](./detector3.rs)
- 测试用例：[Detector3](../../../tests/cases/Detector3)

### 规则4：未使用的private接口
存在未使用的private接口
- 源码：[detector4.rs](./detector4.rs)
- 测试用例：[Detector4](../../../tests/cases/Detector4)

### 规则5：位移运算溢出
位移运算时，保证位移数<被位移数的位数，确保左右位移不移除
- 源码：[detector5.rs](./detector5.rs)
- 测试用例：[Detector5](../../../tests/cases/Detector5)

### 规则6：调用了其他模块已经弃用的函数
调用了其他模块已经弃用的函数，可能导能导致逻辑错误
- 源码：[detector6.rs](./detector6.rs)
- 测试用例：[Detector6](../../../tests/cases/Detector6)
- TODO：待补全已弃用的函数集合

### 规则7：先乘后除
先乘后除，先除后乘可能降低结果精度
- 源码：[detector7.rs](./detector7.rs)
- 测试用例：[Detector7](../../../tests/cases/Detector8)

### 规则8：依赖库未明确版本
依赖库版本应使用版本号或commit号，避免使用分支名
- 源码：[detector8.rs](./detector8.rs)
- 测试用例：[Detector8](../../../tests/cases/Detector8)