# ESLint and Prettier Configuration

This document describes the code quality and formatting setup for the Vue3 frontend.

## ESLint Configuration

### Overview
ESLint is configured to enforce code quality standards for Vue3, TypeScript, and JavaScript files.

### Configuration File
- **Location**: `.eslintrc.cjs`
- **Parser**: `vue-eslint-parser` with `@typescript-eslint/parser` for TypeScript
- **Extends**:
  - `eslint:recommended` - ESLint recommended rules
  - `plugin:@typescript-eslint/recommended` - TypeScript recommended rules
  - `plugin:vue/vue3-recommended` - Vue3 recommended rules

### Custom Rules
```javascript
{
  'vue/multi-word-component-names': 'off',  // Allow single-word component names
  '@typescript-eslint/no-explicit-any': 'warn',  // Warn on 'any' type usage
  '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }]  // Allow unused vars starting with _
}
```

### Ignored Files
The following files and directories are excluded from linting (`.eslintignore`):
- `dist/` - Build output
- `node_modules/` - Dependencies
- `*.config.ts` - Configuration files
- `*.config.js` - Configuration files

### Running ESLint
```bash
# Lint and auto-fix issues
npm run lint

# Lint without fixing
npx eslint . --ext .vue,.ts,.js
```

## Prettier Configuration

### Overview
Prettier is configured to automatically format code for consistent style.

### Configuration File
- **Location**: `.prettierrc.json`

### Settings
```json
{
  "semi": true,              // Use semicolons
  "singleQuote": true,       // Use single quotes
  "tabWidth": 2,             // 2 spaces for indentation
  "trailingComma": "es5",    // Trailing commas where valid in ES5
  "printWidth": 100,         // Line width limit
  "arrowParens": "avoid",    // Omit parens when possible in arrow functions
  "endOfLine": "lf"          // Use LF line endings
}
```

### Running Prettier
```bash
# Format all files
npm run format

# Check formatting without fixing
npx prettier --check "src/**/*.{vue,ts,js,json,css}"
```

## Integration with VS Code

### Recommended Extensions
1. **ESLint** (`dbaeumer.vscode-eslint`)
2. **Prettier - Code formatter** (`esbenp.prettier-vscode`)
3. **Volar** (`Vue.volar`) - Vue3 language support

### VS Code Settings
Add to `.vscode/settings.json`:
```json
{
  "editor.formatOnSave": true,
  "editor.defaultFormatter": "esbenp.prettier-vscode",
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true
  },
  "[vue]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  },
  "[typescript]": {
    "editor.defaultFormatter": "esbenp.prettier-vscode"
  }
}
```

## CI/CD Integration

### Pre-commit Hooks
Consider adding husky and lint-staged for automatic linting before commits:

```bash
npm install --save-dev husky lint-staged
npx husky init
```

Add to `package.json`:
```json
{
  "lint-staged": {
    "*.{vue,ts,js}": [
      "eslint --fix",
      "prettier --write"
    ],
    "*.{json,css}": [
      "prettier --write"
    ]
  }
}
```

### GitHub Actions
Example workflow for CI:
```yaml
- name: Lint
  run: npm run lint
  
- name: Check formatting
  run: npx prettier --check "src/**/*.{vue,ts,js,json,css}"
```

## Common Issues and Solutions

### Issue: TypeScript version warning
**Warning**: "You are currently running a version of TypeScript which is not officially supported"

**Solution**: This is a warning only. The configuration works with TypeScript 5.x. To suppress, you can:
1. Downgrade TypeScript to the supported version
2. Upgrade `@typescript-eslint` packages to support newer TypeScript versions
3. Ignore the warning (functionality is not affected)

### Issue: Unused variables warnings
**Warning**: Variables defined but never used

**Solution**: 
- Prefix unused variables with underscore: `_unusedVar`
- Remove unused imports and variables
- These are warnings, not errors, and won't block builds

### Issue: `any` type warnings
**Warning**: "Unexpected any. Specify a different type"

**Solution**:
- Replace `any` with specific types
- Use `unknown` for truly unknown types
- Use generics for flexible typing
- These are warnings to encourage better type safety

## Maintenance

### Updating Dependencies
```bash
# Update ESLint and plugins
bun update eslint eslint-plugin-vue @typescript-eslint/parser @typescript-eslint/eslint-plugin vue-eslint-parser

# Update Prettier
bun update prettier
```

### Adding New Rules
1. Edit `.eslintrc.cjs` to add or modify rules
2. Run `npm run lint` to test
3. Run `npm run format` to ensure compatibility with Prettier

## Summary

✅ **ESLint**: Configured and working
✅ **Prettier**: Configured and working
✅ **Integration**: Both tools work together seamlessly
✅ **Scripts**: Available via `npm run lint` and `npm run format`

The configuration follows Vue3 and TypeScript best practices while maintaining code quality and consistency.
