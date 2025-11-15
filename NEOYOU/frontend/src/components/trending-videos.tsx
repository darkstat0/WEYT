'use client';

import { useState } from 'react';
import { TrendingUp, Clock, Eye, PlayCircle, ChevronRight } from 'lucide-react';

interface Video {
  id: string;
  title: string;
  thumbnailUrl: string;
  duration: number;
  views: number;
  creator: {
    id: string;
    username: string;
    avatarUrl: string;
  };
}

interface TrendingVideosProps {
  className?: string;
}

export function TrendingVideos({ className = '' }: TrendingVideosProps) {
  const [selectedCategory, setSelectedCategory] = useState('all');
  const [hoveredVideo, setHoveredVideo] = useState<string | null>(null);

  // Mock data for trending videos
  const trendingVideos: Video[] = [
    {
      id: '1',
      title: 'Amazing Nature Documentary - 4K Ultra HD',
      thumbnailUrl: 'https://images.unsplash.com/photo-1506905925346-21bda4d32df4?w=400&h=225&fit=crop',
      duration: 1250,
      views: 2450000,
      creator: {
        id: '1',
        username: 'Nature Channel',
        avatarUrl: 'https://images.unsplash.com/photo-1535713875002-d1d0cf377fde?w=40&h=40&fit=crop&crop=face',
      },
    },
    {
      id: '2',
      title: 'Top 10 Tech Gadgets of 2024',
      thumbnailUrl: 'https://images.unsplash.com/photo-1511707171634-5f897ff02aa9?w=400&h=225&fit=crop',
      duration: 845,
      views: 1870000,
      creator: {
        id: '2',
        username: 'Tech Reviews',
        avatarUrl: 'https://images.unsplash.com/photo-1494790108755-2616b612b786?w=40&h=40&fit=crop&crop=face',
      },
    },
    {
      id: '3',
      title: 'Cooking Masterclass: Italian Pasta',
      thumbnailUrl: 'https://images.unsplash.com/photo-1567620905732-2d1ec7ab7445?w=400&h=225&fit=crop',
      duration: 1560,
      views: 1320000,
      creator: {
        id: '3',
        username: 'Chef Marco',
        avatarUrl: 'https://images.unsplash.com/photo-1500648767791-00dcc994a43e?w=40&h=40&fit=crop&crop=face',
      },
    },
    {
      id: '4',
      title: 'Travel Vlog: Hidden Gems in Japan',
      thumbnailUrl: 'https://images.unsplash.com/photo-1493976040374-85c8e12f0c0e?w=400&h=225&fit=crop',
      duration: 2100,
      views: 980000,
      creator: {
        id: '4',
        username: 'Wanderlust',
        avatarUrl: 'https://images.unsplash.com/photo-1472099645785-5658abf4ff4e?w=40&h=40&fit=crop&crop=face',
      },
    },
    {
      id: '5',
      title: 'Gaming Highlights: Epic Moments',
      thumbnailUrl: 'https://images.unsplash.com/photo-1511512578047-dfb367046420?w=400&h=225&fit=crop',
      duration: 920,
      views: 875000,
      creator: {
        id: '5',
        username: 'GameMaster',
        avatarUrl: 'https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=40&h=40&fit=crop&crop=face',
      },
    },
    {
      id: '6',
      title: 'Fitness Challenge: 30 Day Transformation',
      thumbnailUrl: 'https://images.unsplash.com/photo-1571019613454-1cb2f99b2d8b?w=400&h=225&fit=crop',
      duration: 1780,
      views: 765000,
      creator: {
        id: '6',
        username: 'FitLife',
        avatarUrl: 'https://images.unsplash.com/photo-1573496359142-b8d87734a5a2?w=40&h=40&fit=crop&crop=face',
      },
    },
  ];

  const categories = [
    { id: 'all', label: 'All' },
    { id: 'gaming', label: 'Gaming' },
    { id: 'music', label: 'Music' },
    { id: 'live', label: 'Live' },
    { id: 'news', label: 'News' },
    { id: 'sports', label: 'Sports' },
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
          <TrendingUp className="h-5 w-5 text-red-600" />
          <h2 className="text-lg font-semibold text-white">Trending Now</h2>
        </div>
        <button className="text-sm text-gray-400 hover:text-white transition-colors flex items-center gap-1">
          View all <ChevronRight className="h-4 w-4" />
        </button>
      </div>

      {/* Categories */}
      <div className="flex gap-2 mb-4 overflow-x-auto pb-2">
        {categories.map((category) => (
          <button
            key={category.id}
            onClick={() => setSelectedCategory(category.id)}
            className={`px-3 py-1.5 rounded-full text-sm font-medium whitespace-nowrap transition-colors ${
              selectedCategory === category.id
                ? 'bg-red-600 text-white'
                : 'bg-gray-800 text-gray-300 hover:bg-gray-700'
            }`}
          >
            {category.label}
          </button>
        ))}
      </div>

      {/* Trending Videos */}
      <div className="space-y-3">
        {trendingVideos.map((video, index) => (
          <div
            key={video.id}
            className="flex gap-3 p-2 rounded-lg hover:bg-gray-800 transition-colors cursor-pointer group"
            onMouseEnter={() => setHoveredVideo(video.id)}
            onMouseLeave={() => setHoveredVideo(null)}
          >
            {/* Rank Number */}
            <div className="flex-shrink-0 w-8 h-8 flex items-center justify-center rounded-full bg-gray-800 text-gray-400 font-medium">
              {index + 1}
            </div>

            {/* Thumbnail */}
            <div className="relative flex-shrink-0 w-40 h-22.5 bg-gray-800 rounded overflow-hidden">
              <img
                src={video.thumbnailUrl}
                alt={video.title}
                className="w-full h-full object-cover"
              />
              
              {/* Duration */}
              <div className="absolute bottom-1 right-1 bg-black/80 text-white text-xs px-1 py-0.5 rounded">
                {formatDuration(video.duration)}
              </div>

              {/* Play Button Overlay */}
              {hoveredVideo === video.id && (
                <div className="absolute inset-0 bg-black/60 flex items-center justify-center">
                  <PlayCircle className="h-8 w-8 text-white opacity-80" />
                </div>
              )}
            </div>

            {/* Video Info */}
            <div className="flex-1 min-w-0">
              <h3 className="text-sm font-medium text-gray-900 line-clamp-2 mb-1 group-hover:text-white transition-colors">
                {video.title}
              </h3>
              
              <div className="flex items-center gap-2 text-xs text-gray-500 mb-1">
                <span>{video.creator.username}</span>
              </div>
              
              <div className="flex items-center gap-3 text-xs text-gray-500">
                <span className="flex items-center gap-1">
                  <Eye className="h-3 w-3" />
                  {formatViews(video.views)} views
                </span>
                <span className="flex items-center gap-1">
                  <Clock className="h-3 w-3" />
                  2 days ago
                </span>
              </div>
            </div>
          </div>
        ))}
      </div>

      {/* Trending Insights */}
      <div className="mt-4 pt-4 border-t border-gray-800">
        <h3 className="text-sm font-semibold text-gray-400 mb-2">Trending Insights</h3>
        <div className="text-xs text-gray-500 space-y-1">
          <p>• Gaming content is up 45% this week</p>
          <p>• Live streams peaked at 3.2M concurrent viewers</p>
          <p>• Shorts format engagement increased by 78%</p>
        </div>
      </div>
    </div>
  );
}