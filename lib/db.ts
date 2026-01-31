import Database from "better-sqlite3";
import path from "path";
import fs from "fs";

const dbPath = process.env.DATABASE_PATH || path.join(process.cwd(), "data", "blog.db");

// Singleton database instance
let dbInstance: Database.Database | null = null;

// Initialize database
function getDb(): Database.Database {
  if (!dbInstance) {
    // Ensure the directory exists
    const dbDir = path.dirname(dbPath);
    if (!fs.existsSync(dbDir)) {
      fs.mkdirSync(dbDir, { recursive: true });
    }
    
    dbInstance = new Database(dbPath);
    
    // Create tables if they don't exist
    dbInstance.exec(`
      CREATE TABLE IF NOT EXISTS post_metadata (
        slug TEXT PRIMARY KEY,
        views INTEGER DEFAULT 0,
        updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
      );
      
      CREATE TABLE IF NOT EXISTS app_settings (
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL,
        updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
      );
    `);
    
    // Initialize default settings if they don't exist
    const settingsCheck = dbInstance.prepare("SELECT COUNT(*) as count FROM app_settings WHERE key = 'chat_maintenance'");
    const settingsResult = settingsCheck.get() as { count: number };
    if (settingsResult.count === 0) {
      const initStmt = dbInstance.prepare("INSERT INTO app_settings (key, value) VALUES ('chat_maintenance', 'false')");
      initStmt.run();
    }
  }
  
  return dbInstance;
}

// Get post metadata
export function getPostMetadata(slug: string): { views: number } | null {
  const db = getDb();
  const stmt = db.prepare("SELECT views FROM post_metadata WHERE slug = ?");
  const result = stmt.get(slug) as { views: number } | undefined;
  
  if (!result) {
    // Initialize if doesn't exist
    initializePostMetadata(slug);
    return { views: 0 };
  }
  
  return result;
}

// Initialize post metadata
export function initializePostMetadata(slug: string): void {
  const db = getDb();
  const stmt = db.prepare(
    "INSERT OR IGNORE INTO post_metadata (slug, views) VALUES (?, 0)"
  );
  stmt.run(slug);
}

// Increment view count
export function incrementViews(slug: string): void {
  const db = getDb();
  
  // Initialize if doesn't exist
  initializePostMetadata(slug);
  
  // Increment views
  const stmt = db.prepare(
    "UPDATE post_metadata SET views = views + 1, updated_at = CURRENT_TIMESTAMP WHERE slug = ?"
  );
  stmt.run(slug);
}

// Get all post metadata
export function getAllPostMetadata(): Record<string, { views: number }> {
  const db = getDb();
  const stmt = db.prepare("SELECT slug, views FROM post_metadata");
  const results = stmt.all() as Array<{ slug: string; views: number }>;
  
  const metadata: Record<string, { views: number }> = {};
  results.forEach((row) => {
    metadata[row.slug] = { views: row.views };
  });
  
  return metadata;
}

// Update post metadata views
export function updatePostViews(slug: string, views: number): void {
  const db = getDb();
  
  // Initialize if doesn't exist
  initializePostMetadata(slug);
  
  // Update views
  const stmt = db.prepare(
    "UPDATE post_metadata SET views = ?, updated_at = CURRENT_TIMESTAMP WHERE slug = ?"
  );
  stmt.run(views, slug);
}

// App settings functions
export function getAppSetting(key: string): string | null {
  const db = getDb();
  const stmt = db.prepare("SELECT value FROM app_settings WHERE key = ?");
  const result = stmt.get(key) as { value: string } | undefined;
  return result?.value || null;
}

export function setAppSetting(key: string, value: string): void {
  const db = getDb();
  const stmt = db.prepare(
    "INSERT OR REPLACE INTO app_settings (key, value, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)"
  );
  stmt.run(key, value);
}

export function isChatMaintenanceMode(): boolean {
  const value = getAppSetting("chat_maintenance");
  return value === "true";
}

export function setChatMaintenanceMode(enabled: boolean): void {
  setAppSetting("chat_maintenance", enabled ? "true" : "false");
}
