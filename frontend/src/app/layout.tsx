import "../styles.css";
import React from "react";

export const metadata = {
  title: "AutoShorts",
  description: "Generate viral clips automatically.",
};

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <body>
        {children}
      </body>
    </html>
  );
}
