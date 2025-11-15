'use client';

import { useState } from 'react';
import { Heart, Clock, Eye, MoreHorizontal, Sparkles } from 'lucide-react';

interface Video {
  id: string;
  title: string;
  thumbnailUrl: string;
  duration: number;
  views: number;
  likes: number;
  creator: {
    id: string;
    username: string;
    avatarUrl: string;
  };
}

interface RecommendedVideosProps {
  className?: string;
}

export function RecommendedVideos({ className = '' }: RecommendedVideosProps) {
  const [hoveredVideo, setHoveredVideo] = useState<string | null>(null);

  // Mock data for recommended videos
  const recommendedVideos: Video[] = [
    {
      id: '1',
      title: 'Learn React 19 in 90 Minutes - Complete Tutorial',
      thumbnailUrl: 'https://images.unsplash.com/photo-1627398242454-45a1465c2479?w=400&h=225&fit=crop',
      duration: 5420,
      views: 125000,
      likes: 8500,
      creator: {
        id: '1',
        username: 'CodeMaster',
        avatarUrl: 'https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=40&h=40&fit=crop&crop=face',
      },
    },
    {
      id: '2',
      title: 'Top 10 JavaScript Frameworks for 2024',
      thumbnailUrl: 'https://images.unsplash.com/photo-1555066931-4365d14bab8c?w=400&h=225&fit=crop',
      duration: 890,
      views: 98000,
      likes: 6200,
      creator: {
        id: '2',
        username: 'TechGuru',
        avatarUrl: 'https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?w=40&h=40&fit=crop&crop=face',
      },
    },
    {
      id: '3',
      title: 'Building a Full-Stack App with Next.js 14',
      thumbnailUrl: 'https://images.unsplash.com/photo-1551288049-bebda4e38f71?w=400&h=225&fit=crop',
      duration: 3240,
      views: 76000,
      likes: 4800,
      creator: {
        id: '3',
        username: 'NextJS Pro',
        avatarUrl: 'https://images.unsplash.com/photo-1494790108755-2616b612b786?w=40&h=40&fit=crop&crop=face',
      },
    },
    {
      id: '4',
      title: 'CSS Grid vs Flexbox: When to Use What',
      thumbnailUrl: 'https://images.unsplash.com/photo-1504674900247-0877df9cc836?w=400&h=225&fit=crop',
      duration: 1240,
      views: 54000,
      likes: 3200,
      creator: {
        id: '4',
        username: 'CSS Wizard',
        avatarUrl: 'https://images.unsplash.com/photo-1517841905240-472988babdf9?w=40&h=40&fit=crop&crop=face',
      },
    },
    {
      id: '5',
      title: 'TypeScript Tips & Tricks for Better Code',
      thumbnailUrl: 'https://images.unsplash.com/photo-1516321318423-f06f85e504b3?w=400&h=225&fit=crop',
      duration: 1560,
      views: 42000,
      likes: 2800,
      creator: {
        id: '5',
        username: 'TypeScript Expert',
        avatarUrl: 'https://images.unsplash.com/photo-1507591064344-4c6ce005b128?w=40&h=40&fit=crop&crop=face',
      },
    },
    {
      id: '6',
      title: 'AI-Powered Development Tools You Should Use',
      thumbnailUrl: 'https://images.unsplash.com/photo-1518709268805-4e9042af2176?w=400&h=225&fit=crop',
      duration: 2100,
      views: 38000,
      likes: 2100,
      creator: {
        id: '6',
        username: 'AI Dev',
        avatarUrl: 'https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=40&h=40&fit=crop&crop=face',
      },
    },
  ];

  // Format duration
  const formatDuration = (seconds: number) => {
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = Math.floor(seconds % 60);
    return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
  };

  // Format views
  const formatViews = (views: number) => {
    if (views >= 1000000) {
      return `${(views / 1000000).toFixed(1)}M`;
    } else if (views >= 1000) {
      return `${(views / 1000).toFixed(1)}K`;
    }
    return views.toString();
  };

  return (
    <div className={`${className} bg-gray-900 rounded-lg p-4`}>
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <Sparkles className="h-5 w-5 text-purple-500" />
          <h2 className="text-lg font-semibold text-white">Recommended for You</h2>
        </div>
        <div className="flex items-center gap-2">
          <button className="text-xs text-gray-400 hover:text-white transition-colors">
            Refresh
          </button>
          <button className="text-gray-400 hover:text-white transition-colors">
            <MoreHorizontal className="h-4 w-4" />
          </button>
        </div>
      </div>

      {/* Why Recommended */}
      <div className="mb-4 p-3 bg-gradient-to-r from-purple-900/30 to-blue-900/30 rounded-lg border border-purple-800/30">
        <p className="text-xs text-purple-300">
          Based on your watching history and interests
        </p>
      </div>

      {/* Recommended Videos Grid */}
      <div className="grid grid-cols-2 gap-3">
        {recommendedVideos.map((video) => (
          <div
            key={video.id}
            className="relative group cursor-pointer"
            onMouseEnter={() => setHoveredVideo(video.id)}
            onMouseLeave={() => setHoveredVideo(null)}
          >
            {/* Thumbnail */}
            <div className="relative aspect-video bg-gray-800 rounded-lg overflow-hidden">
              <img
                src={video.thumbnailUrl}
                alt={video.title}
                className="w-full h-full object-cover"
              />
              
              {/* Duration */}
              <div className="absolute bottom-1 right-1 bg-black/80 text-white text-xs px-1 py-0.5 rounded">
                {formatDuration(video.duration)}
              </div>

              {/* Hover Overlay */}
              {hoveredVideo === video.id && (
                <div className="absolute inset-0 bg-black/60 flex items-center justify-center">
                  <div className="bg-white/20 backdrop-blur-sm rounded-full p-2">
                    <svg xmlns="http://www.w3.org/2000/svg" className="h-5 w-5 text-white" viewBox="0 0 20 20" fill="currentColor">
                      <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z" clipRule="evenodd" />
                    </svg>
                  </div>
                </div>
              )}
            </div>
            
            {/* Video Info */}
            <div className="mt-2">
              <h3 className="text-sm font-medium text-gray-900 line-clamp-2 mb-1">
                {video.title}
              </h3>
              
              <div className="flex items-center gap-2 text-xs text-gray-500 mb-1">
                <span>{video.creator.username}</span>
              </div>
              
              <div className="flex items-center justify-between text-xs text-gray-500">
                <span className="flex items-center gap-1">
                  <Eye className="h-3 w-3" />
                  {formatViews(video.views)}
                </span>
                <span className="flex items-center gap-1">
                  <Heart className="h-3 w-3" />
                  {formatViews(video.likes)}
                </span>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* AI Insights */}
      <div className="mt-4 pt-4 border-t border-gray-800">
        <h3 className="text-sm font-semibold text-gray-400 mb-2 flex items-center gap-2">
          <Sparkles className="h-4 w-4 text-purple-500" />
          AI Insights
        </h3>
        <div className="text-xs text-gray-500 space-y-1">
          <p>• 85% of users who watched similar videos completed them</p>
          <p>• Average engagement time: 4:32</p>
          <p>• Content matches your interests: Web Development, AI</p>
        </div>
      </div>

      {/* Recommendation Feedback */}
      <div className="mt-4 flex gap-2">
        <button className="flex-1 text-xs text-gray-400 hover:text-white transition-colors py-1.5 border border-gray-700 rounded hover:bg-gray-800">
          Not Interested
        </button>
        <button className="flex-1 text-xs text-gray-400 hover:text-white transition-colors py-1.5 border border-gray-700 rounded hover:bg-gray-800">
          More Like This
        </button>
      </div>
    </div>
  );
}