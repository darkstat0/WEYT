'use client';

import { useEffect, useState } from 'react';
import { useQuery } from 'react-query';
import { VideoPlayer } from '@/components/video-player';
import { VideoGrid } from '@/components/video-grid';
import { Sidebar } from '@/components/sidebar';
import { Header } from '@/components/header';
import { CreatePost } from '@/components/create-post';
import { TrendingVideos } from '@/components/trending-videos';
import { RecommendedVideos } from '@/components/recommended-videos';
import { toast } from 'react-hot-toast';

interface Video {
  id: string;
  title: string;
  description: string;
  thumbnailUrl: string;
  videoUrl: string;
  duration: number;
  views: number;
  likes: number;
  comments: number;
  createdAt: string;
  creator: {
    id: string;
    username: string;
    avatarUrl: string;
  };
}

export default function HomePage() {
  const [selectedVideo, setSelectedVideo] = useState<Video | null>(null);
  const [showCreatePost, setShowCreatePost] = useState(false);

  // Fetch trending videos
  const { data: trendingVideos, isLoading: trendingLoading } = useQuery(
    'trending-videos',
    async () => {
      const response = await fetch('/api/videos/trending');
      if (!response.ok) throw new Error('Failed to fetch trending videos');
      return response.json();
    }
  );

  // Fetch recommended videos
  const { data: recommendedVideos, isLoading: recommendedLoading } = useQuery(
    'recommended-videos',
    async () => {
      const response = await fetch('/api/videos/recommended');
      if (!response.ok) throw new Error('Failed to fetch recommended videos');
      return response.json();
    }
  );

  // Fetch all videos
  const { data: allVideos, isLoading: allVideosLoading } = useQuery(
    'all-videos',
    async () => {
      const response = await fetch('/api/videos');
      if (!response.ok) throw new Error('Failed to fetch videos');
      return response.json();
    }
  );

  const handleVideoSelect = (video: Video) => {
    setSelectedVideo(video);
  };

  const handleCreatePost = () => {
    setShowCreatePost(true);
  };

  const handleCloseCreatePost = () => {
    setShowCreatePost(false);
    // Refresh videos list
    // In a real app, you'd use react-query's invalidateQueries
  };

  return (
    <div className="min-h-screen bg-black">
      <Header onCreatePost={handleCreatePost} />
      
      <div className="flex">
        <Sidebar />
        
        <main className="flex-1 p-4">
          {showCreatePost && (
            <CreatePost onClose={handleCloseCreatePost} />
          )}
          
          {selectedVideo ? (
            <div className="mb-6">
              <VideoPlayer video={selectedVideo} onBack={() => setSelectedVideo(null)} />
            </div>
          ) : (
            <div className="mb-6">
              <div className="flex items-center justify-between mb-4">
                <h1 className="text-2xl font-bold text-white">Trending Now</h1>
                <button 
                  onClick={handleCreatePost}
                  className="bg-red-600 hover:bg-red-700 text-white px-4 py-2 rounded-lg flex items-center gap-2"
                >
                  <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                    <path fillRule="evenodd" d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" clipRule="evenodd" />
                  </svg>
                  Create
                </button>
              </div>
              
              {trendingLoading ? (
                <div className="flex justify-center items-center h-64">
                  <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-red-600"></div>
                </div>
              ) : (
                <VideoGrid videos={trendingVideos?.data || []} onVideoSelect={handleVideoSelect} />
              )}
            </div>
          )}
          
          <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
            <div className="lg:col-span-2">
              <h2 className="text-xl font-bold text-white mb-4">Recommended for You</h2>
              {recommendedLoading ? (
                <div className="flex justify-center items-center h-64">
                  <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-red-600"></div>
                </div>
              ) : (
                <VideoGrid videos={recommendedVideos?.data || []} onVideoSelect={handleVideoSelect} />
              )}
            </div>
            
            <div>
              <TrendingVideos />
            </div>
          </div>
        </main>
      </div>
    </div>
  );
}