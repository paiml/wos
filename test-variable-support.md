# WOS Bash Variable Support Investigation

## Test Plan

Test if WOS bash supports variable assignments and expansion.

### Manual Test in Browser

1. Open http://localhost:8000/dist/wos/
2. Type these commands in the terminal:
   ```bash
   NAME="World"
   echo "Hello $NAME"
   ```
3. Expected output if variables ARE supported: `Hello World`
4. Actual output if variables are NOT supported: `Hello ` (empty)

### Script File Test

If manual test succeeds, then the problem is with how vim saves files.
If manual test fails, then WOS bash doesn't support variables at all.

## Investigation Results

Will be filled in after manual testing...
