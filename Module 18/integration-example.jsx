import React, { useState, useEffect } from 'react';
import init, { DataProcessor } from './pkg/reentrancy_deadlock_module';

/**
 * Module 18: Re-entrancy Deadlock Demo Component
 * 
 * This React component demonstrates the re-entrancy bug in the Rust WASM module.
 * 
 * THE BUG:
 * When processing items, if the callback tries to call another Rust function
 * (like get_item_count()), it will cause a panic because the RefCell is already
 * borrowed mutably.
 */

export default function ReentrancyDemo() {
    const [processor, setProcessor] = useState(null);
    const [items, setItems] = useState([]);
    const [newItem, setNewItem] = useState('');
    const [log, setLog] = useState([]);
    const [isProcessing, setIsProcessing] = useState(false);

    // Initialize the WASM module
    useEffect(() => {
        async function loadWasm() {
            try {
                await init();
                const proc = new DataProcessor();
                setProcessor(proc);
                addLog('✅ WASM module loaded successfully', 'success');
            } catch (error) {
                addLog(`❌ Failed to load WASM: ${error}`, 'error');
            }
        }
        loadWasm();
    }, []);

    const addLog = (message, type = 'info') => {
        setLog(prev => [...prev, { message, type, timestamp: new Date() }]);
    };

    const handleAddItem = () => {
        if (!processor || !newItem.trim()) return;

        try {
            processor.add_item(newItem);
            setItems(prev => [...prev, newItem]);
            setNewItem('');
            addLog(`✅ Added item: "${newItem}"`, 'success');
        } catch (error) {
            addLog(`❌ Error adding item: ${error}`, 'error');
        }
    };

    const handleAddSampleItems = () => {
        const samples = ['Apple', 'Banana', 'Cherry', 'Date', 'Elderberry'];
        samples.forEach(item => {
            processor.add_item(item);
            setItems(prev => [...prev, item]);
            addLog(`✅ Added: "${item}"`, 'success');
        });
    };

    /**
     * SAFE PROCESSING
     * The callback only uses JavaScript operations and doesn't call back into Rust
     */
    const handleSafeProcess = () => {
        if (!processor) return;

        try {
            setIsProcessing(true);
            addLog('🔄 Starting SAFE processing...', 'info');

            processor.process_items((item, index) => {
                // SAFE: Only JavaScript operations here
                addLog(`  Processing ${index}: ${item}`, 'info');
                console.log(`Processing item ${index}: ${item}`);
            });

            addLog('✅ Safe processing completed!', 'success');
        } catch (error) {
            addLog(`❌ Error: ${error}`, 'error');
        } finally {
            setIsProcessing(false);
        }
    };

    /**
     * BUGGY PROCESSING
     * ⚠️ THIS WILL CAUSE A PANIC! ⚠️
     * 
     * The callback tries to call get_item_count() which attempts to borrow
     * the same RefCell that's already borrowed by process_items().
     * 
     * This triggers: "already borrowed: BorrowMutError"
     */
    const handleBuggyProcess = () => {
        if (!processor) return;

        try {
            setIsProcessing(true);
            addLog('⚠️  Starting BUGGY processing...', 'warning');
            addLog('⚠️  This will trigger a re-entrancy panic!', 'warning');

            processor.process_items((item, index) => {
                addLog(`  Processing ${index}: ${item}`, 'info');
                
                // BUG: This call back into Rust causes the panic!
                addLog('  🐛 Calling get_item_count() from callback...', 'warning');
                
                try {
                    const count = processor.get_item_count(); // ❌ PANIC!
                    addLog(`  Count: ${count}`, 'info');
                } catch (error) {
                    addLog(`💥 PANIC! ${error}`, 'error');
                    throw error;
                }
            });

            // This won't be reached
            addLog('✅ Processing completed', 'success');
        } catch (error) {
            addLog(`💥 RE-ENTRANCY BUG TRIGGERED!`, 'error');
            addLog(`   ${error}`, 'error');
            addLog(`   The callback tried to borrow RefCell while already borrowed!`, 'error');
        } finally {
            setIsProcessing(false);
        }
    };

    /**
     * Another example: Validation with state query
     * This will also trigger the bug if the validator calls back into Rust
     */
    const handleBuggyValidation = () => {
        if (!processor) return;

        try {
            addLog('⚠️  Starting BUGGY validation...', 'warning');

            const isValid = processor.validate_items((item, index) => {
                addLog(`  Validating ${index}: ${item}`, 'info');
                
                // BUG: Trying to get processed count during validation
                try {
                    const processed = processor.get_processed_count(); // ❌ PANIC!
                    addLog(`  Processed so far: ${processed}`, 'info');
                } catch (error) {
                    addLog(`💥 PANIC during validation! ${error}`, 'error');
                    throw error;
                }
                
                return true; // All items are valid
            });

            addLog(`Validation result: ${isValid}`, isValid ? 'success' : 'error');
        } catch (error) {
            addLog(`💥 VALIDATION BUG! ${error}`, 'error');
        }
    };

    const handleGetCount = () => {
        if (!processor) return;

        try {
            const count = processor.get_item_count();
            addLog(`ℹ️  Item count: ${count}`, 'info');
        } catch (error) {
            addLog(`❌ Error: ${error}`, 'error');
        }
    };

    const handleGetSummary = () => {
        if (!processor) return;

        try {
            const summary = processor.get_summary();
            addLog(`ℹ️  ${summary}`, 'info');
        } catch (error) {
            addLog(`❌ Error: ${error}`, 'error');
        }
    };

    const handleClear = () => {
        if (!processor) return;

        try {
            processor.clear();
            setItems([]);
            addLog('✅ All items cleared', 'success');
        } catch (error) {
            addLog(`❌ Error: ${error}`, 'error');
        }
    };

    return (
        <div style={styles.container}>
            <div style={styles.header}>
                <h1>🐛 Module 18: Re-entrancy Deadlock</h1>
                <p>Interactive React Demo</p>
            </div>

            <div style={styles.warning}>
                <h3>⚠️ Warning: Intentional Bug!</h3>
                <p>
                    This demo shows how re-entrancy causes RefCell panics when
                    JavaScript callbacks call back into Rust.
                </p>
            </div>

            {/* Add Items Section */}
            <div style={styles.section}>
                <h2>1. Add Items <span style={styles.badgeSafe}>SAFE</span></h2>
                <div style={styles.controls}>
                    <input
                        type="text"
                        value={newItem}
                        onChange={(e) => setNewItem(e.target.value)}
                        onKeyPress={(e) => e.key === 'Enter' && handleAddItem()}
                        placeholder="Enter item name..."
                        style={styles.input}
                    />
                    <button onClick={handleAddItem} style={styles.button}>
                        Add Item
                    </button>
                    <button onClick={handleAddSampleItems} style={styles.buttonSuccess}>
                        Add 5 Samples
                    </button>
                </div>
                <ul style={styles.itemsList}>
                    {items.map((item, index) => (
                        <li key={index} style={styles.listItem}>
                            Item {index + 1}: {item}
                        </li>
                    ))}
                </ul>
            </div>

            {/* Safe Processing */}
            <div style={styles.section}>
                <h2>2. Safe Processing <span style={styles.badgeSafe}>SAFE</span></h2>
                <button 
                    onClick={handleSafeProcess} 
                    disabled={isProcessing}
                    style={styles.buttonSuccess}
                >
                    Process Items (Safe)
                </button>
            </div>

            {/* Buggy Processing */}
            <div style={styles.section}>
                <h2>3. Buggy Processing <span style={styles.badgeBug}>BUG!</span></h2>
                <button 
                    onClick={handleBuggyProcess} 
                    disabled={isProcessing}
                    style={styles.buttonDanger}
                >
                    ⚠️ Trigger Re-entrancy Bug!
                </button>
                <button 
                    onClick={handleBuggyValidation} 
                    disabled={isProcessing}
                    style={{ ...styles.buttonDanger, marginLeft: '10px' }}
                >
                    ⚠️ Buggy Validation!
                </button>
            </div>

            {/* State Queries */}
            <div style={styles.section}>
                <h2>4. Query State <span style={styles.badgeSafe}>SAFE</span></h2>
                <div style={styles.controls}>
                    <button onClick={handleGetCount} style={styles.button}>
                        Get Count
                    </button>
                    <button onClick={handleGetSummary} style={styles.button}>
                        Get Summary
                    </button>
                    <button onClick={handleClear} style={styles.buttonDanger}>
                        Clear All
                    </button>
                </div>
            </div>

            {/* Console Log */}
            <div style={styles.section}>
                <h2>Console Log</h2>
                <div style={styles.console}>
                    {log.map((entry, index) => (
                        <div key={index} style={{
                            ...styles.logEntry,
                            color: getLogColor(entry.type)
                        }}>
                            [{entry.timestamp.toLocaleTimeString()}] {entry.message}
                        </div>
                    ))}
                </div>
            </div>
        </div>
    );
}

function getLogColor(type) {
    switch (type) {
        case 'error': return '#f48771';
        case 'success': return '#89d185';
        case 'warning': return '#e5c07b';
        case 'info': return '#61afef';
        default: return '#d4d4d4';
    }
}

const styles = {
    container: {
        maxWidth: '1200px',
        margin: '0 auto',
        padding: '20px',
        fontFamily: 'Arial, sans-serif'
    },
    header: {
        textAlign: 'center',
        padding: '30px',
        background: 'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
        color: 'white',
        borderRadius: '8px',
        marginBottom: '20px'
    },
    warning: {
        background: '#fff3cd',
        border: '2px solid #ffc107',
        borderRadius: '6px',
        padding: '15px',
        marginBottom: '20px'
    },
    section: {
        background: '#f9f9f9',
        border: '2px solid #e0e0e0',
        borderRadius: '8px',
        padding: '20px',
        marginBottom: '20px'
    },
    controls: {
        display: 'flex',
        gap: '10px',
        marginBottom: '15px',
        flexWrap: 'wrap'
    },
    input: {
        flex: 1,
        padding: '10px',
        border: '2px solid #ddd',
        borderRadius: '4px',
        fontSize: '14px'
    },
    button: {
        padding: '10px 20px',
        border: 'none',
        borderRadius: '6px',
        fontSize: '14px',
        fontWeight: 'bold',
        cursor: 'pointer',
        background: '#667eea',
        color: 'white'
    },
    buttonSuccess: {
        padding: '10px 20px',
        border: 'none',
        borderRadius: '6px',
        fontSize: '14px',
        fontWeight: 'bold',
        cursor: 'pointer',
        background: '#4facfe',
        color: 'white'
    },
    buttonDanger: {
        padding: '10px 20px',
        border: 'none',
        borderRadius: '6px',
        fontSize: '14px',
        fontWeight: 'bold',
        cursor: 'pointer',
        background: '#f5576c',
        color: 'white'
    },
    itemsList: {
        listStyle: 'none',
        padding: 0
    },
    listItem: {
        padding: '10px',
        background: 'white',
        marginBottom: '5px',
        borderRadius: '4px',
        borderLeft: '4px solid #667eea'
    },
    console: {
        background: '#1e1e1e',
        padding: '15px',
        borderRadius: '6px',
        fontFamily: 'monospace',
        fontSize: '13px',
        maxHeight: '300px',
        overflowY: 'auto'
    },
    logEntry: {
        marginBottom: '5px'
    },
    badgeSafe: {
        background: '#4facfe',
        color: 'white',
        padding: '4px 8px',
        borderRadius: '4px',
        fontSize: '12px',
        marginLeft: '10px'
    },
    badgeBug: {
        background: '#f5576c',
        color: 'white',
        padding: '4px 8px',
        borderRadius: '4px',
        fontSize: '12px',
        marginLeft: '10px'
    }
};
