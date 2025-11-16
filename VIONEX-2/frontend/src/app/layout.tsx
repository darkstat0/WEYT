import { Inter } from 'next/font/google';
import { Providers } from '@/components/providers';
import { Toaster } from '@/components/ui/toaster';
import { Header } from '@/components/layout/header';
import { Footer } from '@/components/layout/footer';
import { ThemeProvider } from '@/components/theme-provider';
import { AuthProvider } from '@/components/auth-provider';
import { Analytics } from '@/components/analytics';
import { SpeedInsights } from '@vercel/speed-insights/next';
import { Metadata } from 'next';

const inter = Inter({ subsets: ['latin'] });

export const metadata: Metadata = {
  metadataBase: new URL(process.env.NEXT_PUBLIC_BASE_URL || 'http://localhost:3000'),
  title: {
    default: 'Video Platform - Next Generation Video Experience',
    template: '%s | Video Platform',
  },
  description: 'Experience the future of video content with AI-powered recommendations, creator tools, and high-quality streaming.',
  keywords: ['video', 'streaming', 'ai', 'creator tools', 'next generation'],
  authors: [{ name: 'Video Platform Team' }],
  creator: 'Video Platform',
  publisher: 'Video Platform',
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
  openGraph: {
    type: 'website',
    locale: 'en_US',
    url: process.env.NEXT_PUBLIC_BASE_URL,
    title: 'Video Platform - Next Generation Video Experience',
    description: 'Experience the future of video content with AI-powered recommendations, creator tools, and high-quality streaming.',
    siteName: 'Video Platform',
    images: [
      {
        url: '/og-image.jpg',
        width: 1200,
        height: 630,
        alt: 'Video Platform',
      },
    ],
  },
  twitter: {
    card: 'summary_large_image',
    title: 'Video Platform - Next Generation Video Experience',
    description: 'Experience the future of video content with AI-powered recommendations, creator tools, and high-quality streaming.',
    images: ['/twitter-image.jpg'],
    creator: '@video_platform',
  },
  icons: {
    icon: '/favicon.ico',
    apple: '/apple-touch-icon.png',
  },
  manifest: '/manifest.json',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" suppressHydrationWarning>
      <head>
        <link rel="preconnect" href="https://fonts.googleapis.com" />
        <link rel="preconnect" href="https://fonts.gstatic.com" crossOrigin="anonymous" />
        <link
          href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700;800&display=swap"
          rel="stylesheet"
        />
        <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.0/css/all.min.css" />
        <Analytics />
        <SpeedInsights />
      </head>
      <body className={`${inter.className} antialiased`} suppressHydrationWarning>
        <ThemeProvider
          attribute="class"
          defaultTheme="system"
          enableSystem
          disableTransitionOnChange
        >
          <Providers>
            <div className="relative flex min-h-screen flex-col bg-background">
              <Header />
              <main className="flex-1">{children}</main>
              <Footer />
            </div>
            <Toaster />
          </Providers>
        </ThemeProvider>
      </body>
    </html>
  );
}