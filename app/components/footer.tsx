export default function Footer() {
  return (
    <footer className="border-t border-border bg-bg-light/80 backdrop-blur-sm mt-auto">
      <div className="mx-auto max-w-6xl px-6 py-8">
        <div className="flex flex-col items-center justify-center gap-4 text-center">
          <p className="text-sm text-text-muted">
            © {new Date().getFullYear()} Vitor de Oliveira
          </p>
          <a
            href="mailto:vitordeoliveira50@gmail.com"
            className="text-sm font-medium text-text-muted hover:text-action-primary transition-colors"
          >
            vitordeoliveira50@gmail.com
          </a>
        </div>
      </div>
    </footer>
  );
}

