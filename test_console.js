// 在浏览器控制台运行此脚本来测试 SQL 执行

// 测试 1: 列出数据库
async function testListDatabases() {
    console.log('=== 测试列出数据库 ===');
    try {
        const result = await window.__TAURI__.core.invoke('list_databases');
        console.log('✓ 成功:', result);
        return result;
    } catch (error) {
        console.error('✗ 失败:', error);
        return null;
    }
}

// 测试 2: 执行 SQL
async function testExecuteSql(database, sql) {
    console.log('=== 测试执行 SQL ===');
    console.log('数据库:', database);
    console.log('SQL:', sql);
    try {
        const result = await window.__TAURI__.core.invoke('execute_sql', {
            database: database,
            sql: sql
        });
        console.log('✓ 成功:', result);
        return result;
    } catch (error) {
        console.error('✗ 失败:', error);
        return null;
    }
}

// 测试 3: 获取表数据
async function testGetTableData(database, table) {
    console.log('=== 测试获取表数据 ===');
    console.log('数据库:', database);
    console.log('表:', table);
    try {
        const result = await window.__TAURI__.core.invoke('get_table_data', {
            database: database,
            table: table,
            page: 1,
            pageSize: 10
        });
        console.log('✓ 成功:', result);
        return result;
    } catch (error) {
        console.error('✗ 失败:', error);
        return null;
    }
}

// 运行所有测试
async function runAllTests() {
    console.log('========================================');
    console.log('开始测试');
    console.log('========================================\n');
    
    // 测试 1
    await testListDatabases();
    console.log('\n');
    
    // 测试 2 - 使用你的数据库名称
    await testExecuteSql('草稿1', 'SELECT * FROM employees LIMIT 5;');
    console.log('\n');
    
    // 测试 3
    await testGetTableData('草稿1', 'employees');
    console.log('\n');
    
    console.log('========================================');
    console.log('测试完成');
    console.log('========================================');
}

// 导出函数供使用
console.log('测试函数已加载。使用方法：');
console.log('1. runAllTests() - 运行所有测试');
console.log('2. testListDatabases() - 测试列出数据库');
console.log('3. testExecuteSql("草稿1", "SELECT 1") - 测试执行 SQL');
console.log('4. testGetTableData("草稿1", "employees") - 测试获取表数据');
