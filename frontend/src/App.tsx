import React, { useState, useEffect } from 'react';
import { exportDatabase, importDatabase, listDatabases, checkHealth, getExportDirPath, ApiResponse } from './api';
import { open } from '@tauri-apps/plugin-dialog';

function App() {
  const [databases, setDatabases] = useState<string[]>([]);
  const [selectedDatabase, setSelectedDatabase] = useState('');
  const [selectedFilePath, setSelectedFilePath] = useState<string>('');
  const [importDatabaseName, setImportDatabaseName] = useState('');
  const [message, setMessage] = useState<{ type: 'success' | 'error'; text: string } | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [serverStatus, setServerStatus] = useState<'online' | 'offline' | 'checking'>('checking');
  const [exportDir, setExportDir] = useState<string>('');

  useEffect(() => {
    checkServerHealth();
    loadDatabases();
    loadExportDir();
  }, []);

  const loadExportDir = async () => {
    try {
      const dir = await getExportDirPath();
      setExportDir(dir);
    } catch (error) {
      console.error('Failed to load export directory:', error);
    }
  };

  const checkServerHealth = async () => {
    try {
      const response = await checkHealth();
      setServerStatus(response.success ? 'online' : 'offline');
    } catch (error) {
      setServerStatus('offline');
    }
  };

  const loadDatabases = async () => {
    try {
      const response: ApiResponse<string[]> = await listDatabases();
      if (response.success && response.data) {
        setDatabases(response.data);
      }
    } catch (error) {
      showMessage('error', 'Failed to load databases');
    }
  };

  const showMessage = (type: 'success' | 'error', text: string) => {
    setMessage({ type, text });
    setTimeout(() => setMessage(null), 5000);
  };

  const handleExport = async () => {
    if (!selectedDatabase) {
      showMessage('error', 'Please select a database to export');
      return;
    }

    setIsLoading(true);
    try {
      const response = await exportDatabase(selectedDatabase);
      if (response.success) {
        showMessage('success', response.message);
      } else {
        showMessage('error', response.message);
      }
    } catch (error) {
      showMessage('error', 'Export failed');
    } finally {
      setIsLoading(false);
    }
  };

  const handleSelectFile = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'SQL Files',
          extensions: ['sql']
        }]
      });
      
      if (selected && typeof selected === 'string') {
        setSelectedFilePath(selected);
      }
    } catch (error) {
      showMessage('error', 'Failed to open file dialog');
    }
  };

  const handleImport = async () => {
    if (!selectedFilePath || !importDatabaseName) {
      showMessage('error', 'Please select a file and enter a database name');
      return;
    }

    setIsLoading(true);
    try {
      const response = await importDatabase(selectedFilePath, importDatabaseName);
      if (response.success) {
        showMessage('success', response.message);
        setImportDatabaseName('');
        setSelectedFilePath('');
        loadDatabases();
      } else {
        showMessage('error', response.message);
      }
    } catch (error) {
      showMessage('error', 'Import failed');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div style={{ maxWidth: '800px', margin: '0 auto', padding: '20px', fontFamily: 'Arial, sans-serif' }}>
      <header style={{ marginBottom: '30px', borderBottom: '2px solid #eee', paddingBottom: '20px' }}>
        <h1 style={{ color: '#333', margin: 0 }}>PostgreSQL Database Tool</h1>
        <div style={{ marginTop: '10px', display: 'flex', alignItems: 'center', gap: '10px' }}>
          <span style={{ fontWeight: 'bold' }}>Server Status:</span>
          <span style={{
            padding: '4px 12px',
            borderRadius: '12px',
            fontSize: '12px',
            backgroundColor: serverStatus === 'online' ? '#d4edda' : serverStatus === 'offline' ? '#f8d7da' : '#fff3cd',
            color: serverStatus === 'online' ? '#155724' : serverStatus === 'offline' ? '#721c24' : '#856404'
          }}>
            {serverStatus === 'online' ? '● Online' : serverStatus === 'offline' ? '● Offline' : '● Checking...'}
          </span>
          <button 
            onClick={() => { checkServerHealth(); loadDatabases(); }}
            style={{ padding: '4px 8px', fontSize: '12px', cursor: 'pointer' }}
          >
            Refresh
          </button>
        </div>
      </header>

      {message && (
        <div style={{
          padding: '12px',
          marginBottom: '20px',
          borderRadius: '4px',
          backgroundColor: message.type === 'success' ? '#d4edda' : '#f8d7da',
          color: message.type === 'success' ? '#155724' : '#721c24',
          border: `1px solid ${message.type === 'success' ? '#c3e6cb' : '#f5c6cb'}`
        }}>
          {message.text}
        </div>
      )}

      <div style={{ display: 'grid', gap: '30px' }}>
        <section style={{ padding: '20px', border: '1px solid #ddd', borderRadius: '8px', backgroundColor: '#fafafa' }}>
          <h2 style={{ color: '#333', marginTop: 0 }}>Export Database</h2>
          <div style={{ marginBottom: '15px' }}>
            <label style={{ display: 'block', marginBottom: '8px', fontWeight: 'bold', color: '#555' }}>
              Select Database:
            </label>
            <select
              value={selectedDatabase}
              onChange={(e) => setSelectedDatabase(e.target.value)}
              style={{
                width: '100%',
                padding: '10px',
                border: '1px solid #ccc',
                borderRadius: '4px',
                fontSize: '14px'
              }}
            >
              <option value="">-- Select a database --</option>
              {databases.map(db => (
                <option key={db} value={db}>{db}</option>
              ))}
            </select>
          </div>
          {exportDir && (
            <div style={{ marginBottom: '15px', fontSize: '12px', color: '#666' }}>
              Export directory: {exportDir}
            </div>
          )}
          <button
            onClick={handleExport}
            disabled={isLoading || !selectedDatabase}
            style={{
              padding: '10px 20px',
              backgroundColor: isLoading ? '#ccc' : '#007bff',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: isLoading ? 'not-allowed' : 'pointer',
              fontSize: '14px',
              fontWeight: 'bold'
            }}
          >
            {isLoading ? 'Exporting...' : 'Export Database'}
          </button>
        </section>

        <section style={{ padding: '20px', border: '1px solid #ddd', borderRadius: '8px', backgroundColor: '#fafafa' }}>
          <h2 style={{ color: '#333', marginTop: 0 }}>Import Database</h2>
          <div style={{ marginBottom: '15px' }}>
            <label style={{ display: 'block', marginBottom: '8px', fontWeight: 'bold', color: '#555' }}>
              Database Name:
            </label>
            <input
              type="text"
              value={importDatabaseName}
              onChange={(e) => setImportDatabaseName(e.target.value)}
              placeholder="Enter database name"
              style={{
                width: '100%',
                padding: '10px',
                border: '1px solid #ccc',
                borderRadius: '4px',
                fontSize: '14px',
                boxSizing: 'border-box'
              }}
            />
          </div>
          <div style={{ marginBottom: '15px' }}>
            <label style={{ display: 'block', marginBottom: '8px', fontWeight: 'bold', color: '#555' }}>
              Select SQL File:
            </label>
            <div style={{ display: 'flex', gap: '10px' }}>
              <button
                onClick={handleSelectFile}
                style={{
                  padding: '10px 20px',
                  backgroundColor: '#6c757d',
                  color: 'white',
                  border: 'none',
                  borderRadius: '4px',
                  cursor: 'pointer',
                  fontSize: '14px'
                }}
              >
                Browse Files
              </button>
              <input
                type="text"
                value={selectedFilePath}
                readOnly
                placeholder="No file selected"
                style={{
                  flex: 1,
                  padding: '10px',
                  border: '1px solid #ccc',
                  borderRadius: '4px',
                  fontSize: '14px',
                  backgroundColor: '#f8f9fa',
                  boxSizing: 'border-box'
                }}
              />
            </div>
          </div>
          <button
            onClick={handleImport}
            disabled={isLoading || !selectedFilePath || !importDatabaseName}
            style={{
              padding: '10px 20px',
              backgroundColor: isLoading ? '#ccc' : '#28a745',
              color: 'white',
              border: 'none',
              borderRadius: '4px',
              cursor: isLoading ? 'not-allowed' : 'pointer',
              fontSize: '14px',
              fontWeight: 'bold'
            }}
          >
            {isLoading ? 'Importing...' : 'Import Database'}
          </button>
        </section>

        <section style={{ padding: '20px', border: '1px solid #ddd', borderRadius: '8px', backgroundColor: '#fafafa' }}>
          <h2 style={{ color: '#333', marginTop: 0 }}>Available Databases</h2>
          {databases.length > 0 ? (
            <ul style={{ listStyle: 'none', padding: 0, margin: 0 }}>
              {databases.map(db => (
                <li key={db} style={{
                  padding: '8px 12px',
                  marginBottom: '4px',
                  backgroundColor: 'white',
                  border: '1px solid #e0e0e0',
                  borderRadius: '4px',
                  fontSize: '14px'
                }}>
                  {db}
                </li>
              ))}
            </ul>
          ) : (
            <p style={{ color: '#666', fontStyle: 'italic' }}>No databases found</p>
          )}
        </section>
      </div>
    </div>
  );
}

export default App;
