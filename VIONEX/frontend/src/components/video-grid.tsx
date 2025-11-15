'use client';

import { useState } from 'react';
import { Image as ImageIcon, Film, Calendar, Eye, Heart, MessageCircle, MoreHorizontal } from 'lucide-react';

interface Video {
  id: string;
  title: string;
  description: string;
  thumbnailUrl: string;
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

interface VideoGridProps {
  videos: Video[];
  loading?: boolean;
  onLoadMore?: () => void;
  hasMore?: boolean;
}

export function VideoGrid({ videos, loading = false, onLoadMore, hasMore = false }: VideoGridProps) {
  const [hoveredVideo, setHoveredVideo] = useState<string | null>(null);

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

  // Format date
  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    const now = new Date();
    const diffTime = Math.abs(now.getTime() - date.getTime());
    const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));
    
    if (diffDays === 1) return '1 day ago';
    if (diffDays < 7) return `${diffDays} days ago`;
    if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
    if (diffDays < 365) return `${Math.floor(diffDays / 30)} months ago`;
    return `${Math.floor(diffDays / 365)} years ago`;
  };

  const handleLoadMore = () => {
    if (onLoadMore && !loading && hasMore) {
      onLoadMore();
    }
  };

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
      {videos.map((video) => (
        <div
          key={video.id}
          className="relative group cursor-pointer"
          onMouseEnter={() => setHoveredVideo(video.id)}
          onMouseLeave={() => setHoveredVideo(null)}
        >
          {/* Thumbnail */}
          <div className="relative aspect-video bg-gray-900 rounded-lg overflow-hidden">
            <img
              src={video.thumbnailUrl}
              alt={video.title}
              className="w-full h-full object-cover transition-transform duration-300 group-hover:scale-105"
            />
            
            {/* Duration Overlay */}
            <div className="absolute bottom-2 right-2 bg-black/80 text-white text-xs px-1.5 py-0.5 rounded">
              {formatDuration(video.duration)}
            </div>
            
            {/* Hover Overlay */}
            {hoveredVideo === video.id && (
              <div className="absolute inset-0 bg-black/40 flex items-center justify-center">
                <div className="bg-white/20 backdrop-blur-sm rounded-full p-3">
                  <Film className="h-6 w-6 text-white" />
                </div>
              </div>
            )}
            
            {/* Video Actions Overlay */}
            {hoveredVideo === video.id && (
              <div className="absolute inset-0 bg-black/60 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity duration-300">
                <div className="flex gap-4">
                  <button className="bg-white/20 backdrop-blur-sm rounded-full p-2 hover:bg-white/30 transition-colors">
                    <Eye className="h-5 w-5 text-white" />
                  </button>
                  <button className="bg-white/20 backdrop-blur-sm rounded-full p-2 hover:bg-white/30 transition-colors">
                    <Heart className="h-5 w-5 text-white" />
                  </button>
                  <button className="bg-white/20 backdrop-blur-sm rounded-full p-2 hover:bg-white/30 transition-colors">
                    <MessageCircle className="h-5 w-5 text-white" />
                  </button>
                </div>
              </div>
            )}
          </div>
          
          {/* Video Info */}
          <div className="mt-3">
            {/* Creator Avatar */}
            <div className="flex items-start gap-2">
              <img
                src={video.creator.avatarUrl}
                alt={video.creator.username}
                className="w-8 h-8 rounded-full flex-shrink-0"
              />
              
              {/* Video Details */}
              <div className="flex-1 min-w-0">
                {/* Title */}
                <h3 className="text-sm font-medium text-gray-900 line-clamp-2 mb-1">
                  {video.title}
                </h3>
                
                {/* Creator Name */}
                <p className="text-xs text-gray-500 mb-1">
                  {video.creator.username}
                </p>
                
                {/* Video Stats */}
                <div className="flex items-center gap-3 text-xs text-gray-500">
                  <span className="flex items-center gap-1">
                    <Eye className="h-3 w-3" />
                    {formatViews(video.views)}
                  </span>
                  <span className="flex items-center gap-1">
                    <Calendar className="h-3 w-3" />
                    {formatDate(video.createdAt)}
                  </span>
                </div>
              </div>
              
              {/* More Button */}
              <button className="text-gray-400 hover:text-gray-600 transition-colors">
                <MoreHorizontal className="h-4 w-4" />
              </button>
            </div>
          </div>
        </div>
      ))}
      
      {/* Loading State */}
      {loading && (
        Array.from({ length: 8 }).map((_, index) => (
          <div key={index} className="aspect-video bg-gray-900 rounded-lg animate-pulse">
            <div className="w-full h-full flex items-center justify-center">
              <ImageIcon className="h-12 w-12 text-gray-700" />
            </div>
          </div>
        ))
      )}
      
      {/* Load More Button */}
      {hasMore && !loading && (
        <div className="col-span-full flex justify-center mt-6">
          <button
            onClick={handleLoadMore}
            className="px-6 py-2 bg-gray-900 text-white rounded-full hover:bg-gray-800 transition-colors"
          >
            Load More Videos
          </button>
        </div>
      )}
      
      {/* No Videos Message */}
      {videos.length === 0 && !loading && (
        <div className="col-span-full flex flex-col items-center justify-center py-12">
          <ImageIcon className="h-16 w-16 text-gray-400 mb-4" />
          <h3 className="text-lg font-medium text-gray-900 mb-2">No videos found</h3>
          <p className="text-gray-500 text-center max-w-md">
            Try adjusting your search or browse more videos to find something interesting.
          </p>
        </div>
      )}
    </div>
  );
}