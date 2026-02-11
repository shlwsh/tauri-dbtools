# Requirements Document

## Introduction

本文档定义了为现有数据库资源管理器添加高级功能的需求，包括SQL执行、表设计和数据编辑功能。这些功能将使用户能够在基于Tauri的桌面应用中执行SQL查询、可视化设计数据库表结构，以及直接编辑表数据。设计参考VSCode数据库插件（如SQLTools、PostgreSQL）的用户体验。

## Glossary

- **SQL_Editor**: 提供SQL语法高亮和查询编辑功能的组件
- **Query_Executor**: 在后端执行SQL语句的Rust服务
- **Result_Renderer**: 以适当格式（表格、消息、错误）渲染查询结果的组件
- **Table_Designer**: 允许用户创建和修改表结构的可视化设计器组件
- **Schema_Editor**: 用于编辑表字段、约束和索引的界面组件
- **Data_Grid**: 显示和编辑表数据的可编辑网格组件
- **Connection_Manager**: 管理数据库连接的服务
- **Query_History**: 存储和管理历史查询的服务
- **Syntax_Highlighter**: 为SQL代码提供语法高亮的组件
- **Auto_Completer**: 提供SQL关键字和数据库对象自动完成的组件
- **Transaction_Manager**: 管理数据库事务的后端服务
- **Schema_Validator**: 验证表结构定义的服务
- **DDL_Generator**: 根据表设计生成DDL语句的服务

## Requirements

### Requirement 1: SQL编辑器界面

**User Story:** 作为数据库开发者，我想要一个功能完善的SQL编辑器，以便高效地编写和执行SQL查询。

#### Acceptance Criteria

1. WHEN用户打开SQL编辑器，THE SQL_Editor SHALL显示一个带有行号的代码编辑区域
2. WHEN用户在编辑器中输入SQL代码，THE Syntax_Highlighter SHALL实时高亮SQL关键字、字符串、注释和数字
3. WHEN用户输入SQL关键字的前几个字符，THE Auto_Completer SHALL显示匹配的关键字建议列表
4. WHEN用户输入表名或列名的前几个字符，THE Auto_Completer SHALL从当前数据库加载并显示匹配的对象名称
5. WHEN用户按下Tab键，THE SQL_Editor SHALL接受当前选中的自动完成建议
6. WHEN用户按下Ctrl+Space，THE SQL_Editor SHALL手动触发自动完成功能
7. WHEN用户选择多行代码并按下Tab键，THE SQL_Editor SHALL增加选中行的缩进
8. WHEN用户选择多行代码并按下Shift+Tab键，THE SQL_Editor SHALL减少选中行的缩进
9. WHEN用户按下Ctrl+/（或Cmd+/），THE SQL_Editor SHALL切换选中行的注释状态

### Requirement 2: SQL查询执行

**User Story:** 作为数据库管理员，我想要执行SQL查询并查看结果，以便检索和管理数据。

#### Acceptance Criteria

1. WHEN用户按下执行按钮或Ctrl+Enter，THE Query_Executor SHALL执行编辑器中的所有SQL语句
2. WHEN用户选中部分SQL代码并按下Ctrl+Enter，THE Query_Executor SHALL仅执行选中的SQL语句
3. WHEN执行SELECT查询，THE Query_Executor SHALL返回结果集和执行时间
4. WHEN执行INSERT、UPDATE或DELETE语句，THE Query_Executor SHALL返回受影响的行数
5. WHEN执行CREATE、ALTER或DROP语句，THE Query_Executor SHALL返回执行成功消息
6. WHEN编辑器包含多个用分号分隔的SQL语句，THE Query_Executor SHALL按顺序执行每个语句
7. IF SQL执行失败，THEN THE Query_Executor SHALL返回数据库错误消息和错误位置信息
8. WHEN查询正在执行时，THE SQL_Editor SHALL显示加载指示器并禁用执行按钮
9. WHEN用户点击取消按钮，THE Query_Executor SHALL尝试取消正在执行的查询

### Requirement 3: 查询结果显示

**User Story:** 作为数据库用户，我想要以清晰的格式查看查询结果，以便理解SQL操作的结果。

#### Acceptance Criteria

1. WHEN SELECT查询返回数据，THE Result_Renderer SHALL在数据表格中显示结果，包含列标题
2. WHEN查询返回零行，THE Result_Renderer SHALL显示"查询返回0行"的消息
3. WHEN显示查询结果，THE Result_Renderer SHALL在列标题中显示列名和数据类型
4. WHEN结果包含NULL值，THE Result_Renderer SHALL以灰色斜体"NULL"文本显示
5. WHEN结果包含长文本，THE Result_Renderer SHALL截断显示并提供悬停工具提示显示完整内容
6. WHEN查询返回超过1000行，THE Result_Renderer SHALL实现虚拟滚动以保持性能
7. WHEN查询执行完成，THE Result_Renderer SHALL显示执行时间和返回的行数
8. WHEN执行多个查询，THE Result_Renderer SHALL为每个查询创建单独的结果标签页
9. WHEN用户右键点击结果单元格，THE Result_Renderer SHALL显示复制选项的上下文菜单

### Requirement 4: 查询历史管理

**User Story:** 作为数据库开发者，我想要访问之前执行的查询，以便重用和参考历史SQL语句。

#### Acceptance Criteria

1. WHEN用户执行SQL查询，THE Query_History SHALL将查询文本、执行时间和数据库名称保存到本地存储
2. WHEN用户打开查询历史面板，THE Query_History SHALL显示最近100条查询的列表
3. WHEN显示历史查询，THE Query_History SHALL显示查询的前50个字符、执行时间和数据库名称
4. WHEN用户点击历史查询项，THE SQL_Editor SHALL将该查询加载到编辑器中
5. WHEN用户在历史面板中搜索，THE Query_History SHALL过滤显示包含搜索文本的查询
6. WHEN用户右键点击历史查询项，THE Query_History SHALL显示删除和复制选项
7. WHEN用户删除历史查询项，THE Query_History SHALL从本地存储中移除该项
8. WHEN历史查询超过100条，THE Query_History SHALL自动删除最旧的查询

### Requirement 5: 表设计器界面

**User Story:** 作为数据库设计师，我想要通过可视化界面创建和修改表结构，以便无需手写DDL即可管理数据库模式。

#### Acceptance Criteria

1. WHEN用户打开表设计器创建新表，THE Table_Designer SHALL显示空白的表设计表单
2. WHEN用户在表设计器中，THE Table_Designer SHALL提供输入表名的字段
3. WHEN用户在表设计器中，THE Table_Designer SHALL提供选择模式（schema）的下拉列表
4. WHEN用户添加列，THE Schema_Editor SHALL显示列定义表单，包含名称、类型、可空性和默认值字段
5. WHEN用户选择列数据类型，THE Schema_Editor SHALL提供PostgreSQL数据类型的下拉列表（INTEGER、VARCHAR、TEXT、TIMESTAMP等）
6. WHEN用户选择VARCHAR或CHAR类型，THE Schema_Editor SHALL提供输入长度的字段
7. WHEN用户选择NUMERIC或DECIMAL类型，THE Schema_Editor SHALL提供输入精度和小数位数的字段
8. WHEN用户定义列，THE Schema_Editor SHALL提供复选框标记列为主键
9. WHEN用户定义列，THE Schema_Editor SHALL提供复选框标记列为唯一约束
10. WHEN用户定义列，THE Schema_Editor SHALL提供输入默认值表达式的字段

### Requirement 6: 约束和索引管理

**User Story:** 作为数据库架构师，我想要定义表约束和索引，以便确保数据完整性和查询性能。

#### Acceptance Criteria

1. WHEN用户在表设计器中，THE Table_Designer SHALL提供"约束"标签页用于管理约束
2. WHEN用户添加主键约束，THE Schema_Editor SHALL允许选择一个或多个列作为主键
3. WHEN用户添加外键约束，THE Schema_Editor SHALL允许选择引用表和引用列
4. WHEN用户添加外键约束，THE Schema_Editor SHALL提供选择ON DELETE和ON UPDATE操作的下拉列表
5. WHEN用户添加检查约束，THE Schema_Editor SHALL提供输入检查表达式的文本字段
6. WHEN用户在表设计器中，THE Table_Designer SHALL提供"索引"标签页用于管理索引
7. WHEN用户添加索引，THE Schema_Editor SHALL允许选择一个或多个列
8. WHEN用户添加索引，THE Schema_Editor SHALL提供选择索引类型（B-tree、Hash、GiST、GIN）的下拉列表
9. WHEN用户添加索引，THE Schema_Editor SHALL提供复选框标记索引为唯一索引

### Requirement 7: DDL生成和执行

**User Story:** 作为数据库管理员，我想要从表设计生成并执行DDL语句，以便将设计应用到数据库。

#### Acceptance Criteria

1. WHEN用户在表设计器中点击"预览SQL"，THE DDL_Generator SHALL生成CREATE TABLE语句
2. WHEN生成CREATE TABLE语句，THE DDL_Generator SHALL包含所有列定义、数据类型和约束
3. WHEN生成CREATE TABLE语句，THE DDL_Generator SHALL包含主键、外键和检查约束定义
4. WHEN生成CREATE TABLE语句，THE DDL_Generator SHALL包含索引创建语句
5. WHEN用户编辑现有表，THE DDL_Generator SHALL生成ALTER TABLE语句以反映更改
6. WHEN用户在表设计器中点击"应用"，THE Query_Executor SHALL执行生成的DDL语句
7. IF DDL执行成功，THEN THE Table_Designer SHALL显示成功消息并关闭设计器
8. IF DDL执行失败，THEN THE Table_Designer SHALL显示错误消息并保持设计器打开以允许修改
9. WHEN用户点击"保存为脚本"，THE Table_Designer SHALL将生成的DDL保存到文件

### Requirement 8: 编辑现有表结构

**User Story:** 作为数据库维护者，我想要修改现有表的结构，以便适应不断变化的业务需求。

#### Acceptance Criteria

1. WHEN用户在数据库浏览器中右键点击表并选择"设计表"，THE Table_Designer SHALL加载该表的当前结构
2. WHEN加载现有表，THE Table_Designer SHALL显示所有现有列及其属性
3. WHEN加载现有表，THE Table_Designer SHALL显示所有现有约束
4. WHEN加载现有表，THE Table_Designer SHALL显示所有现有索引
5. WHEN用户修改列属性，THE Schema_Editor SHALL标记该列为已修改
6. WHEN用户添加新列，THE Schema_Editor SHALL标记该列为新增
7. WHEN用户删除列，THE Schema_Editor SHALL标记该列为待删除并显示确认对话框
8. WHEN用户删除被外键引用的列，THE Schema_Validator SHALL显示警告消息
9. WHEN用户重命名列，THE DDL_Generator SHALL生成ALTER TABLE RENAME COLUMN语句

### Requirement 9: 内联数据编辑

**User Story:** 作为数据管理员，我想要直接在结果网格中编辑表数据，以便快速更新记录而无需编写UPDATE语句。

#### Acceptance Criteria

1. WHEN查询结果显示在Data_Grid中，THE Data_Grid SHALL允许用户双击单元格进入编辑模式
2. WHEN用户编辑单元格，THE Data_Grid SHALL根据列的数据类型显示适当的输入控件
3. WHEN列类型为BOOLEAN，THE Data_Grid SHALL显示复选框
4. WHEN列类型为DATE或TIMESTAMP，THE Data_Grid SHALL显示日期选择器
5. WHEN列类型为TEXT或VARCHAR，THE Data_Grid SHALL显示文本输入框
6. WHEN用户修改单元格值，THE Data_Grid SHALL在行首显示修改指示器
7. WHEN用户按下Enter或Tab键，THE Data_Grid SHALL保存单元格更改并移动到下一个单元格
8. WHEN用户按下Escape键，THE Data_Grid SHALL取消单元格编辑并恢复原值
9. WHEN用户点击"保存更改"按钮，THE Transaction_Manager SHALL为所有修改的行生成UPDATE语句

### Requirement 10: 批量数据修改

**User Story:** 作为数据操作员，我想要批量修改多行数据，以便高效地更新大量记录。

#### Acceptance Criteria

1. WHEN用户修改多行数据，THE Data_Grid SHALL跟踪所有修改的行
2. WHEN用户点击"保存更改"，THE Transaction_Manager SHALL在单个事务中执行所有UPDATE语句
3. IF任何UPDATE失败，THEN THE Transaction_Manager SHALL回滚所有更改并显示错误消息
4. WHEN所有UPDATE成功，THE Data_Grid SHALL刷新显示的数据并清除修改指示器
5. WHEN用户点击"放弃更改"，THE Data_Grid SHALL恢复所有修改的单元格到原始值
6. WHEN用户关闭包含未保存更改的标签页，THE Data_Grid SHALL显示确认对话框
7. WHEN表没有主键，THE Data_Grid SHALL禁用内联编辑并显示警告消息
8. WHEN用户选择多行并按下Delete键，THE Data_Grid SHALL显示确认对话框以删除选中的行

### Requirement 11: 数据验证

**User Story:** 作为数据质量管理员，我想要在保存前验证数据，以便防止无效数据进入数据库。

#### Acceptance Criteria

1. WHEN用户编辑单元格，THE Data_Grid SHALL验证输入是否符合列的数据类型
2. WHEN用户输入无效的数字格式，THE Data_Grid SHALL显示内联错误消息
3. WHEN用户输入无效的日期格式，THE Data_Grid SHALL显示内联错误消息
4. WHEN用户尝试在NOT NULL列中输入NULL，THE Data_Grid SHALL显示错误消息
5. WHEN用户输入超过VARCHAR长度限制的文本，THE Data_Grid SHALL显示错误消息
6. WHEN用户输入违反检查约束的值，THE Data_Grid SHALL在保存时显示数据库错误
7. WHEN用户输入违反唯一约束的值，THE Data_Grid SHALL在保存时显示数据库错误
8. WHEN存在验证错误，THE Data_Grid SHALL禁用"保存更改"按钮

### Requirement 12: 添加和删除行

**User Story:** 作为数据录入员，我想要在数据网格中添加和删除行，以便管理表记录。

#### Acceptance Criteria

1. WHEN用户点击"添加行"按钮，THE Data_Grid SHALL在网格底部插入新的空行
2. WHEN添加新行，THE Data_Grid SHALL为具有默认值的列预填充默认值
3. WHEN添加新行，THE Data_Grid SHALL为自增主键列显示"(自动)"占位符
4. WHEN用户填写新行并点击"保存更改"，THE Transaction_Manager SHALL生成INSERT语句
5. WHEN用户选择一行或多行并点击"删除行"，THE Data_Grid SHALL显示确认对话框
6. WHEN用户确认删除，THE Transaction_Manager SHALL生成DELETE语句
7. WHEN删除操作成功，THE Data_Grid SHALL从网格中移除已删除的行
8. IF删除操作因外键约束失败，THEN THE Data_Grid SHALL显示约束违反错误消息

### Requirement 13: 与数据库浏览器集成

**User Story:** 作为应用用户，我想要SQL编辑器与现有数据库浏览器无缝集成，以便获得一致的体验。

#### Acceptance Criteria

1. WHEN数据库已连接，THE SQL_Editor SHALL使用当前Connection_Manager中的数据库连接
2. WHEN没有数据库连接，THE SQL_Editor SHALL显示提示用户连接数据库的消息
3. WHEN用户在数据库浏览器中右键点击表，THE SQL_Editor SHALL提供"生成SELECT查询"选项
4. WHEN用户选择"生成SELECT查询"，THE SQL_Editor SHALL创建新的SQL标签页并插入SELECT语句
5. WHEN用户在数据库浏览器中右键点击表，THE Table_Designer SHALL提供"设计表"选项
6. WHEN用户选择"设计表"，THE Table_Designer SHALL打开该表的设计器
7. WHEN通过Table_Designer修改表结构，THE数据库浏览器SHALL刷新以显示更新的模式
8. WHEN用户在数据库浏览器中切换数据库，THE SQL_Editor SHALL更新以使用新的数据库连接

### Requirement 14: 错误处理和用户反馈

**User Story:** 作为数据库用户，我想要在操作失败时获得清晰的反馈，以便理解和解决问题。

#### Acceptance Criteria

1. IF SQL执行因语法错误失败，THEN THE Result_Renderer SHALL高亮显示错误位置（如果数据库提供）
2. IF SQL执行因权限错误失败，THEN THE Result_Renderer SHALL显示权限错误消息
3. IF在执行期间数据库连接丢失，THEN THE Query_Executor SHALL显示连接错误并尝试重新连接
4. WHEN长时间运行的查询正在执行，THE SQL_Editor SHALL显示进度指示器和经过的时间
5. WHEN用户取消查询，THE Query_Executor SHALL向数据库发送取消请求
6. IF查询取消成功，THEN THE Result_Renderer SHALL显示"查询已取消"消息
7. WHEN数据验证在内联编辑期间失败，THE Data_Grid SHALL在单元格级别显示内联错误消息
8. WHEN DDL执行失败，THE Table_Designer SHALL显示详细的错误消息和失败的SQL语句

### Requirement 15: 性能和响应性

**User Story:** 作为应用用户，我想要应用保持响应，即使处理大型结果集，以便获得流畅的用户体验。

#### Acceptance Criteria

1. WHEN查询返回超过1000行，THE Result_Renderer SHALL使用虚拟滚动仅渲染可见行
2. WHEN用户滚动大型结果集，THE Result_Renderer SHALL在100毫秒内渲染新的可见行
3. WHEN加载表结构，THE Table_Designer SHALL在后台异步加载约束和索引信息
4. WHEN用户在SQL_Editor中输入，THE Syntax_Highlighter SHALL在50毫秒内更新高亮
5. WHEN用户触发自动完成，THE Auto_Completer SHALL在200毫秒内显示建议
6. WHEN执行查询，THE Query_Executor SHALL在Rust后端的单独线程中执行以避免阻塞UI
7. WHEN保存多行更改，THE Transaction_Manager SHALL使用批处理语句以提高性能

### Requirement 16: 数据安全和完整性

**User Story:** 作为系统管理员，我想要SQL编辑器维护数据完整性和安全性，以便用户不会意外损坏数据或违反安全策略。

#### Acceptance Criteria

1. WHEN执行DML语句，THE Transaction_Manager SHALL在事务中包装操作
2. WHEN事务失败，THE Transaction_Manager SHALL回滚所有更改并保持数据一致性
3. WHEN用户尝试执行DROP或TRUNCATE语句，THE SQL_Editor SHALL显示破坏性操作的确认对话框
4. WHEN用户确认破坏性操作，THE Query_Executor SHALL记录操作到应用日志
5. WHEN显示敏感数据，THE Result_Renderer SHALL遵守任何应用级数据屏蔽策略
6. WHEN保存查询历史，THE Query_History SHALL不存储包含密码或敏感信息的查询
7. WHEN执行多个语句，THE Query_Executor SHALL在第一个错误处停止执行

### Requirement 17: 键盘快捷键

**User Story:** 作为高级用户，我想要使用键盘快捷键，以便更高效地操作应用。

#### Acceptance Criteria

1. WHEN用户按下Ctrl+Enter（或Cmd+Enter），THE SQL_Editor SHALL执行当前查询
2. WHEN用户按下Ctrl+Shift+Enter，THE SQL_Editor SHALL执行所有查询
3. WHEN用户按下Ctrl+K，THE SQL_Editor SHALL清空编辑器内容
4. WHEN用户按下Ctrl+S，THE SQL_Editor SHALL保存当前查询到查询历史
5. WHEN用户按下Ctrl+H，THE SQL_Editor SHALL打开查询历史面板
6. WHEN用户按下F5，THE Data_Grid SHALL刷新当前显示的数据
7. WHEN用户按下Ctrl+F，THE SQL_Editor SHALL打开查找对话框
8. WHEN用户按下Ctrl+R，THE SQL_Editor SHALL打开替换对话框

### Requirement 18: 导出查询结果

**User Story:** 作为数据分析师，我想要导出查询结果，以便在其他工具中使用数据。

#### Acceptance Criteria

1. WHEN用户右键点击查询结果，THE Result_Renderer SHALL显示"导出"选项
2. WHEN用户选择导出，THE Result_Renderer SHALL提供CSV、JSON和Excel格式选项
3. WHEN用户选择CSV格式，THE Result_Renderer SHALL生成包含标题行的CSV文件
4. WHEN用户选择JSON格式，THE Result_Renderer SHALL生成JSON数组，每行作为一个对象
5. WHEN用户选择Excel格式，THE Result_Renderer SHALL生成包含格式化列的XLSX文件
6. WHEN导出大型结果集，THE Result_Renderer SHALL显示进度指示器
7. WHEN导出完成，THE Result_Renderer SHALL打开文件保存对话框

