"use client";

import { useState, useEffect } from "react";
import { useRouter, useParams } from "next/navigation";
import Link from "next/link";
import { updatePostMetadataViews, getPostMetadataForAdmin } from "@/app/actions";

export default function EditPostViewsPage() {
  const router = useRouter();
  const params = useParams();
  const slug = params.slug as string;
  
  const [views, setViews] = useState<number>(0);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState("");
  const [success, setSuccess] = useState(false);

  useEffect(() => {
    async function loadPost() {
      try {
        const result = await getPostMetadataForAdmin(slug);
        if (result.success) {
          setViews(result.views);
        } else {
          setError(result.error || "Failed to load post metadata");
        }
      } catch (err) {
        setError("Failed to load post metadata");
      } finally {
        setLoading(false);
      }
    }
    
    loadPost();
  }, [slug]);

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    setError("");
    setSuccess(false);
    setSaving(true);

    const result = await updatePostMetadataViews(slug, views);
    
    if (result.success) {
      setSuccess(true);
      setTimeout(() => {
        router.push("/admin/dashboard");
      }, 1500);
    } else {
      setError(result.error || "Failed to update views");
      setSaving(false);
    }
  }

  if (loading) {
    return (
      <div className="relative min-h-screen bg-bg flex items-center justify-center">
        <div className="bg-pattern"></div>
        <div className="relative text-text">Loading...</div>
      </div>
    );
  }

  return (
    <div className="relative min-h-screen bg-bg">
      <div className="bg-pattern"></div>
      
      <main className="relative mx-auto max-w-2xl px-6 py-16">
        <div className="mb-6">
          <Link
            href="/admin/dashboard"
            className="text-action-primary hover:underline text-sm"
          >
            ← Back to Dashboard
          </Link>
        </div>
        
        <div className="rounded-lg border border-border bg-bg-light p-8">
          <h1 className="text-3xl font-bold text-text mb-2">Edit Post Views</h1>
          <p className="text-text-muted mb-6">
            Post: <code className="bg-bg-dark px-2 py-1 rounded text-sm">{slug}</code>
          </p>
          
          <form onSubmit={handleSubmit} className="space-y-6">
            <div>
              <label htmlFor="views" className="block text-sm font-medium text-text mb-2">
                Views
              </label>
              <input
                id="views"
                type="number"
                min="0"
                value={views}
                onChange={(e) => setViews(parseInt(e.target.value) || 0)}
                required
                className="w-full px-4 py-2 rounded-lg border border-border bg-bg text-text focus:outline-none focus:ring-2 focus:ring-action-primary focus:border-transparent"
                disabled={saving}
              />
            </div>
            
            {error && (
              <div className="p-3 rounded-lg bg-alert-danger/10 border border-alert-danger/20 text-alert-danger text-sm">
                {error}
              </div>
            )}
            
            {success && (
              <div className="p-3 rounded-lg bg-alert-success/10 border border-alert-success/20 text-alert-success text-sm">
                Views updated successfully! Redirecting...
              </div>
            )}
            
            <div className="flex gap-4">
              <button
                type="submit"
                disabled={saving}
                className="px-6 py-2 rounded-lg bg-action-primary text-white font-medium hover:opacity-90 transition-opacity disabled:opacity-50 disabled:cursor-not-allowed"
              >
                {saving ? "Saving..." : "Save Changes"}
              </button>
              
              <Link
                href="/admin/dashboard"
                className="px-6 py-2 rounded-lg border border-border text-text font-medium hover:bg-bg-dark transition-colors"
              >
                Cancel
              </Link>
            </div>
          </form>
        </div>
      </main>
    </div>
  );
}

