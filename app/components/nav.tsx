import Link from "next/link";
import ThemeToggle from "./theme-toggle";

export default function Nav() {
  return (
    <nav className="sticky top-0 z-50 border-b border-border bg-bg-light/80 backdrop-blur-sm">
      <div className="mx-auto max-w-6xl px-6 py-5">
        <div className="flex items-center justify-between">
          <Link 
            href="/" 
            className="text-2xl font-bold text-text hover:text-action-primary transition-colors tracking-tight"
          >
            vitor<span className="text-action-primary">.ws</span>
          </Link>
          <div className="flex items-center gap-8">
            <Link
              href="/"
              className="text-sm font-medium text-text-muted hover:text-action-primary transition-colors relative group"
            >
              Home
              <span className="absolute -bottom-1 left-0 w-0 h-0.5 bg-action-primary transition-all group-hover:w-full"></span>
            </Link>
            <Link
              href="/blog"
              className="text-sm font-medium text-text-muted hover:text-action-primary transition-colors relative group"
            >
              Blog
              <span className="absolute -bottom-1 left-0 w-0 h-0.5 bg-action-primary transition-all group-hover:w-full"></span>
            </Link>
            <div className="h-6 w-px bg-border"></div>
            <ThemeToggle />
          </div>
        </div>
      </div>
    </nav>
  );
}

