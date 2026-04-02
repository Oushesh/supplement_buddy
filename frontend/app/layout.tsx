import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = {
  title: "Supplement Buddy",
  description: "Find the best supplements using AI-powered search",
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body className="min-h-screen bg-gray-50 text-gray-900">{children}</body>
    </html>
  );
}
