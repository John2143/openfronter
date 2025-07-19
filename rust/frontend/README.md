# OpenFronter Frontend

A React frontend application for OpenFronter, built with Vite and Bun, managed with Nix for reproducible builds.

## Prerequisites

- [Nix](https://nixos.org/download.html) with flakes support
- [Bun](https://bun.sh/) (optional, as it's provided through Nix)

## Development Setup

### Using Nix Flake (Recommended)

1. **Enter the development shell:**
   ```bash
   nix develop
   ```

   This will provide you with:
   - Bun runtime
   - TypeScript and TypeScript Language Server
   - ESLint
   - bun2nix tool

2. **Install dependencies:**
   ```bash
   bun install
   ```

3. **Start development server:**
   ```bash
   bun run dev
   # or
   npm run dev
   ```

   This will start the Vite development server on `http://localhost:3000`.

### Using bun2nix

This project uses `bun2nix` to generate Nix expressions from the Bun lockfile, enabling reproducible builds.

#### Updating Dependencies

When you add or update dependencies:

1. **Update package.json and install:**
   ```bash
   bun add <package-name>
   # or
   bun remove <package-name>
   ```

2. **Generate new Nix expressions:**
   ```bash
   bun run generate-nix
   # or directly:
   bun2nix
   ```

   This will update `bun.nix` and `bun-lockfile.nix` files.

3. **Commit the changes:**
   ```bash
   git add package.json bun.lockb bun.nix bun-lockfile.nix
   git commit -m "Update dependencies and regenerate Nix files"
   ```

#### Manual bun2nix Generation

If you need to manually regenerate the Nix files:

```bash
# Ensure you have a clean bun.lockb
bun install

# Generate Nix expressions
bun2nix

# The following files will be updated/created:
# - bun.nix (dependency expressions)
# - bun-lockfile.nix (lockfile as Nix expression)
```

## Building

### Development Build
```bash
bun run build
```

### Production Build with Nix
```bash
nix build
```

The built artifacts will be in `./result/` (symlink to Nix store).

## Available Scripts

- `bun run dev` - Start development server
- `bun run build` - Build for production
- `bun run preview` - Preview production build locally
- `bun run start` - Alias for dev
- `bun run serve` - Alias for preview
- `bun run generate-nix` - Generate Nix expressions from lockfile

## Project Structure

```
frontend/
├── src/                     # Source code
├── extra_assets/           # Additional assets
├── dist/                   # Build output (created by Vite)
├── package.json            # Package dependencies and scripts
├── bun.lockb              # Bun lockfile (binary)
├── bun.nix                # Generated Nix dependency expressions
├── bun-lockfile.nix       # Generated Nix lockfile expression
├── default.nix            # Nix derivation for building the app
├── flake.nix              # Nix flake configuration
├── flake.lock             # Nix flake lockfile
└── vite.config.ts         # Vite configuration
```

## Nix Configuration

The project uses a Nix flake with the following inputs:
- `nixpkgs` - Main package repository
- `flake-utils` - Utility functions for flakes
- `bun2nix` - Tool for converting Bun lockfiles to Nix

### Custom Build Process

The `default.nix` defines a custom build process that:
1. Uses `mkBunDerivation` to handle Bun dependencies
2. Runs `bun run build` to create Vite production build
3. Installs static files and extra assets to the Nix store

## Troubleshooting

### Dependencies Out of Sync

If you encounter issues with dependencies:

1. Clean and reinstall:
   ```bash
   rm -rf node_modules bun.lockb
   bun install
   ```

2. Regenerate Nix files:
   ```bash
   bun2nix
   ```

3. Update flake lock if needed:
   ```bash
   nix flake update
   ```

### Build Issues

If the Nix build fails:

1. Ensure all dependencies are properly declared in `package.json`
2. Check that `bun.nix` and `bun-lockfile.nix` are up to date
3. Verify the build works locally with `bun run build`

## Contributing

1. Make your changes
2. Ensure dependencies are up to date and Nix files are regenerated
3. Test the build with `nix build`
4. Submit a pull request

## License

MIT - See the project root for license details.
