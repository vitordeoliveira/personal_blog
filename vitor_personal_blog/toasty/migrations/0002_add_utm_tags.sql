-- Visits from before these columns keep only their utm_source.
ALTER TABLE "page_views" ADD COLUMN "utm_content" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "page_views" ADD COLUMN "utm_medium" TEXT;
-- #[toasty::breakpoint]
ALTER TABLE "page_views" ADD COLUMN "utm_campaign" TEXT;
