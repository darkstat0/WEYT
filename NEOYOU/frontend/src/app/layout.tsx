import type { Metadata } from 'next';
import { Inter } from 'next/font/google';
import './globals.css';
import { Header } from '@/components/header';
import { Sidebar } from '@/components/sidebar';
import { CreatePost } from '@/components/create-post';
import { useState } from 'react';

const inter = Inter({ subsets: ['latin'] });

export const metadata: Metadata = {
  title: 'NeoYou - Next Generation Video Platform',
  description: 'Experience the future of video content with AI-powered recommendations, creator tools, and high-quality streaming.',
  keywords: ['video platform', 'AI', 'creator tools', 'streaming', 'entertainment'],
  authors: [{ name: 'NeoYou Team' }],
  viewport: 'width=device-width, initial-scale=1',
  openGraph: {
    title: 'NeoYou - Next Generation Video Platform',
    description: 'Experience the future of video content with AI-powered recommendations, creator tools, and high-quality streaming.',
    url: 'https://neoyou.com',
    siteName: 'NeoYou',
    images: [
      {
        url: '/og-image.jpg',
        width: 1200,
        height: 630,
        alt: 'NeoYou Platform',
      },
    ],
    locale: 'en_US',
    type: 'website',
  },
  twitter: {
    card: 'summary_large_image',
    title: 'NeoYou - Next Generation Video Platform',
    description: 'Experience the future of video content with AI-powered recommendations, creator tools, and high-quality streaming.',
    images: ['/twitter-image.jpg'],
  },
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      'max-video-preview': -1,
      'max-image-preview': 'large',
      'max-snippet': -1,
    },
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  const [isCreatePostOpen, setIsCreatePostOpen] = useState(false);
  const [isSidebarOpen, setIsSidebarOpen] = useState(true);

  return (
    <html lang="en">
      <body className={inter.className}>
        <div className="flex h-screen bg-black text-white overflow-hidden">
          {/* Sidebar */}
          <aside className={`${isSidebarOpen ? 'translate-x-0' : '-translate-x-full'} md:translate-x-0 fixed md:static inset-y-0 left-0 z-40 w-64 bg-gray-900 transition-transform duration-300 ease-in-out`}>
            <Sidebar />
          </aside>

          {/* Main Content */}
          <div className="flex-1 flex flex-col overflow-hidden">
            {/* Header */}
            <Header 
              onMenuToggle={() => setIsSidebarOpen(!isSidebarOpen)} 
            />

            {/* Page Content */}
            <main className="flex-1 overflow-y-auto">
              {children}
            </main>
          </div>
        </div>

        {/* Create Post Modal */}
        <CreatePost 
          isOpen={isCreatePostOpen} 
          onClose={() => setIsCreatePostOpen(false)} 
        />
      </body>
    </html>
  );
}