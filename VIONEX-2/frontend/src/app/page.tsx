import { Suspense } from 'react';
import { HeroSection } from '@/components/sections/hero';
import { TrendingSection } from '@/components/sections/trending';
import { CategoriesSection } from '@/components/sections/categories';
import { CreatorsSection } from '@/components/sections/creators';
import { FeaturesSection } from '@/components/sections/features';
import { LoadingSpinner } from '@/components/ui/loading-spinner';
import { VideoPlayer } from '@/components/video/player';
import { AIAssistant } from '@/components/ai/assistant';

export default function HomePage() {
  return (
    <main className="min-h-screen bg-background">
      <Suspense fallback={<LoadingSpinner />}>
        <HeroSection />
      </Suspense>
      
      <Suspense fallback={<LoadingSpinner />}>
        <TrendingSection />
      </Suspense>
      
      <Suspense fallback={<LoadingSpinner />}>
        <CategoriesSection />
      </Suspense>
      
      <Suspense fallback={<LoadingSpinner />}>
        <CreatorsSection />
      </Suspense>
      
      <Suspense fallback={<LoadingSpinner />}>
        <FeaturesSection />
      </Suspense>
      
      {/* AI Assistant Widget */}
      <AIAssistant />
      
      {/* Demo Video Player */}
      <div className="container mx-auto px-4 py-8">
        <h2 className="text-3xl font-bold mb-6">Featured Video</h2>
        <div className="aspect-video bg-black rounded-lg overflow-hidden">
          <VideoPlayer
            src="/sample-video.mp4"
            poster="/sample-thumbnail.jpg"
            controls={true}
            autoplay={false}
          />
        </div>
      </div>
    </main>
  );
}