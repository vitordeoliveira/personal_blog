import { redirect } from "next/navigation";
import { isAdminAuthenticated } from "@/app/actions";
import LoginForm from "./login-form";

export const dynamic = 'force-dynamic';

export default async function AdminLoginPage() {
  const isAuthenticated = await isAdminAuthenticated();
  
  if (isAuthenticated) {
    redirect("/admin/dashboard");
  }

  return (
    <div className="relative min-h-screen bg-bg flex items-center justify-center">
      <div className="bg-pattern"></div>
      
      <div className="relative w-full max-w-md mx-auto px-6">
        <LoginForm />
      </div>
    </div>
  );
}

