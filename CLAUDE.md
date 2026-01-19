# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Next.js 16 blog application built with TypeScript, React 19, Tailwind CSS 4, and SQLite (better-sqlite3). The blog features markdown-based content, view tracking, an admin dashboard, and an integrated AI sales agent chat interface.

## Development Commands

### Package Manager
This project uses **pnpm** as the package manager.

### Common Commands
```bash
# Development server (runs on http://localhost:3000)
pnpm dev

# Build for production
pnpm build

# Start production server
pnpm start

# Lint code
pnpm lint
```

### Database Setup
The SQLite database is automatically initialized on first run. The database file is stored at `data/blog.db` by default (configurable via `DATABASE_PATH` environment variable).

### Native Dependencies
The project uses `better-sqlite3` which requires native compilation. When deploying or on fresh installs, you may need to rebuild:
```bash
pnpm exec node-gyp rebuild --directory node_modules/better-sqlite3
```

## Architecture

### Application Structure

**App Router (Next.js 16)**: Uses the App Router with server components by default.
- `app/page.tsx` - Home page with recent blog posts and AI chat button
- `app/blog/page.tsx` - Blog listing page
- `app/blog/[slug]/page.tsx` - Individual blog post pages
- `app/admin/` - Admin dashboard for managing posts (protected by cookie-based auth)
- `app/components/` - Shared React components
- `app/actions.ts` - Server actions for view tracking, auth, and AI agent integration

**Data Layer**:
- `lib/db.ts` - SQLite database singleton and post metadata operations (views, timestamps)
- `lib/posts.ts` - Markdown post parsing, gray-matter frontmatter processing, and remark HTML conversion
- `posts/` - Markdown blog posts with frontmatter metadata
- `data/blog.db` - SQLite database (created at runtime)

**Blog Post Workflow**:
1. Posts are stored as markdown files in `posts/` directory
2. Frontmatter metadata includes: `title`, `subtitle`, `description`, `date`, `tags`, and `ready` (publish status)
3. Posts are only visible when `ready: true` in frontmatter
4. Post views are tracked in SQLite and increment on page visits
5. Admin can view unpublished posts and manually adjust view counts

### Design System

The application uses a custom OKLCH-based design system defined in `.cursor/rules/base_design.mdc`. Key principles:

- **OKLCH Color Space**: All colors use `oklch(lightness chroma hue)` format for perceptual uniformity
- **CSS Custom Properties**: Colors defined as CSS variables in `app/globals.css`
- **Tailwind Integration**: Uses `@theme` directive to map CSS variables to Tailwind classes
- **Dark Mode**: Supports both system preference (media query) and manual toggle (data attribute)

**Color Categories**:
- Neutral colors (backgrounds, text, borders): `chroma: 0`
- Action colors (interactive elements): blue/purple hues with moderate chroma
- Alert colors (status indicators): red/yellow/green/blue semantic colors

**Usage Pattern**:
- Background: `bg-bg`, `bg-bg-light`, `bg-bg-dark`
- Text: `text-text`, `text-text-muted`
- Borders: `border-border`, `border-border-muted`
- Actions: `bg-action-primary`, `bg-action-secondary`
- Alerts: `bg-alert-danger`, `bg-alert-warning`, `bg-alert-success`, `bg-alert-info`

### Authentication

Simple cookie-based admin authentication:
- Credentials set via environment variables (`ADMIN_USERNAME`, `ADMIN_PASSWORD`)
- Session cookie stored client-side with 7-day expiration
- Server actions use `requireAdmin()` to protect admin routes
- No user registration or multiple user support

### AI Sales Agent Integration

The blog integrates with an external sales agent API (see `API_DOCUMENTATION.md`):
- Environment variables: `SALES_AGENT_API_KEY` and `SALES_AGENT_URL`
- Server actions: `getAgent()` and `chatWithAgent()` in `app/actions.ts`
- UI: `ChatButton` and `ChatModal` components provide chat interface
- The agent responds using LangChain with configured personality

### Dynamic Rendering

Pages that display view counts use `export const dynamic = 'force-dynamic'` to ensure real-time data fetching instead of static generation.

## Configuration Files

- `tsconfig.json` - TypeScript configuration with `@/*` path alias for root directory
- `next.config.ts` - Next.js configuration (minimal, uses defaults)
- `postcss.config.mjs` - PostCSS configuration for Tailwind CSS
- `eslint.config.mjs` - ESLint configuration using flat config format
- `nixpacks.toml` - Deployment configuration for Nixpacks/Railway with native dependency build steps

## Environment Variables

Required for full functionality:
```bash
# Admin authentication
ADMIN_USERNAME=admin
ADMIN_PASSWORD=admin

# Database path (optional, defaults to data/blog.db)
DATABASE_PATH=data/blog.db

# Sales agent API integration
SALES_AGENT_API_KEY=sk_...
SALES_AGENT_URL=https://agents.invisible-matrix.com
```

## Important Notes

### SQLite Considerations
- Uses `better-sqlite3` (synchronous) for simplicity
- Database is a single file, backed up by version control via `.db` file
- Singleton pattern prevents multiple database connections
- Auto-initializes tables on first connection

### Post Publishing
- Posts MUST have `ready: true` in frontmatter to be visible on public pages
- Admin dashboard shows all posts regardless of `ready` status
- Use `getAllPosts()` for public pages and `getAllPostsForAdmin()` for admin

### Server Actions
All data mutations (view tracking, auth, admin operations, AI chat) use Next.js Server Actions with `"use server"` directive. These are defined in `app/actions.ts`.

### Markdown Processing
Posts are processed using:
- `gray-matter` for frontmatter parsing
- `remark` with `remark-html` plugin for markdown-to-HTML conversion
- HTML is rendered using `dangerouslySetInnerHTML` (sanitize if accepting user-generated content)

### Styling Guidelines
- Follow the OKLCH color system defined in `.cursor/rules/base_design.mdc`
- Use CSS variables for all colors (never hardcode colors)
- Test dark mode compatibility for all new UI components
- Use Tailwind utility classes mapped from CSS variables
