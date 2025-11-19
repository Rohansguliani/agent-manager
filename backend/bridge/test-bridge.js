#!/usr/bin/env node

/**
 * Test script for gemini-bridge.js
 * 
 * Tests the bridge script functionality:
 * 1. Invalid JSON handling
 * 2. Missing fields validation
 * 3. Valid message handling (requires auth)
 * 4. Process lifecycle
 */

import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const BRIDGE_SCRIPT = join(__dirname, 'gemini-bridge.js');

function runTest(name, input, expectedPattern, timeout = 5000) {
  return new Promise((resolve, reject) => {
    console.log(`\n🧪 Test: ${name}`);
    
    const proc = spawn('node', [BRIDGE_SCRIPT], {
      stdio: ['pipe', 'pipe', 'pipe'],
    });

    let output = '';
    let errorOutput = '';

    proc.stdout.on('data', (data) => {
      output += data.toString();
    });

    proc.stderr.on('data', (data) => {
      errorOutput += data.toString();
    });

    proc.on('close', (code) => {
      const fullOutput = output + errorOutput;
      
      // Extract JSON lines from output (ignore non-JSON stderr)
      const jsonLines = output.split('\n').filter(line => {
        try {
          JSON.parse(line);
          return true;
        } catch {
          return false;
        }
      });
      
      // Get the last JSON response (most relevant)
      const lastJson = jsonLines.length > 0 ? jsonLines[jsonLines.length - 1] : '';
      const lastJsonObj = lastJson ? JSON.parse(lastJson) : null;
      
      // Check if output matches expected pattern
      // For error patterns, check if JSON contains the expected message or pattern
      const matchesPattern = fullOutput.includes(expectedPattern) || 
        (lastJsonObj && lastJsonObj.message && lastJsonObj.message.includes(expectedPattern)) ||
        (lastJsonObj && lastJsonObj.status === expectedPattern) ||
        (expectedPattern === 'error' && code !== 0 && lastJsonObj && lastJsonObj.status === 'error') ||
        (expectedPattern === 'success' && code === 0 && lastJsonObj && lastJsonObj.status === 'success');
      
      if (matchesPattern) {
        console.log(`✅ PASS`);
        resolve(true);
      } else {
        console.log(`❌ FAIL`);
        console.log(`   Expected pattern: ${expectedPattern}`);
        console.log(`   Last JSON: ${lastJson.substring(0, 200)}`);
        console.log(`   Full output: ${fullOutput.substring(0, 300)}`);
        console.log(`   Exit code: ${code}`);
        reject(new Error(`Test failed: ${name}`));
      }
    });

    // Write input to stdin
    if (input) {
      proc.stdin.write(input);
      proc.stdin.end();
    }

    // Timeout
    setTimeout(() => {
      proc.kill();
      reject(new Error(`Test timeout: ${name}`));
    }, timeout);
  });
}

async function runAllTests() {
  console.log('=== Testing Gemini Bridge Script ===\n');
  
  const tests = [
    // Test 1: Invalid JSON
    {
      name: 'Invalid JSON',
      input: 'invalid json\n',
      expectedPattern: 'Invalid JSON',
    },
    
    // Test 2: Missing type field
    {
      name: 'Missing type field',
      input: '{"content": "test"}\n',
      expectedPattern: 'missing "type" field',
    },
    
    // Test 3: Unknown request type
    {
      name: 'Unknown request type',
      input: '{"type": "unknown"}\n',
      expectedPattern: 'Unknown request type',
    },
    
    // Test 4: Missing content
    {
      name: 'Missing content',
      input: '{"type": "message"}\n',
      expectedPattern: 'Invalid request',
    },
    
    // Test 5: Empty content
    {
      name: 'Empty content',
      input: '{"type": "message", "content": ""}\n',
      expectedPattern: 'Invalid request',
    },
  ];

  let passed = 0;
  let failed = 0;

  for (const test of tests) {
    try {
      await runTest(test.name, test.input, test.expectedPattern);
      passed++;
    } catch (error) {
      failed++;
      console.error(`   Error: ${error.message}`);
    }
  }

  console.log(`\n=== Test Results ===`);
  console.log(`✅ Passed: ${passed}`);
  console.log(`❌ Failed: ${failed}`);
  console.log(`📊 Total: ${tests.length}`);

  if (failed > 0) {
    process.exit(1);
  }
}

// Run tests
runAllTests().catch((error) => {
  console.error('Test runner error:', error);
  process.exit(1);
});

