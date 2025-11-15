'use client';

import { useState, useRef, useEffect } from 'react';
import ReactPlayer from 'react-player';
import { Play, Pause, Volume2, VolumeX, SkipBack, SkipForward, Settings, Fullscreen } from 'lucide-react';

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

interface VideoPlayerProps {
  video: Video;
  onBack: () => void;
}

export function VideoPlayer({ video, onBack }: VideoPlayerProps) {
  const [playing, setPlaying] = useState(false);
  const [volume, setVolume] = useState(0.8);
  const [muted, setMuted] = useState(false);
  const [played, setPlayed] = useState(0);
  const [loaded, setLoaded] = useState(0);
  const [duration, setDuration] = useState(0);
  const [showControls, setShowControls] = useState(true);
  const [quality, setQuality] = useState('auto');
  const [playbackRate, setPlaybackRate] = useState(1);
  const [isFullscreen, setIsFullscreen] = useState(false);
  
  const playerRef = useRef<ReactPlayer>(null);
  const controlsTimeoutRef = useRef<NodeJS.Timeout>();

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

  // Handle play/pause
  const handlePlayPause = () => {
    setPlaying(!playing);
  };

  // Handle volume change
  const handleVolumeChange = (newVolume: number) => {
    setVolume(newVolume);
    setMuted(newVolume === 0);
  };

  // Handle mute/unmute
  const handleMute = () => {
    setMuted(!muted);
  };

  // Handle seek
  const handleSeek = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!playerRef.current) return;
    
    const rect = e.currentTarget.getBoundingClientRect();
    const pos = (e.clientX - rect.left) / rect.width;
    playerRef.current.seekTo(pos);
  };

  // Handle progress
  const handleProgress = (progress: { played: number; playedSeconds: number }) => {
    setPlayed(progress.played);
  };

  // Handle duration
  const handleDuration = (duration: number) => {
    setDuration(duration);
  };

  // Handle ready
  const handleReady = () => {
    setLoaded(1);
  };

  // Handle controls visibility
  const handleMouseMove = () => {
    setShowControls(true);
    if (controlsTimeoutRef.current) {
      clearTimeout(controlsTimeoutRef.current);
    }
    controlsTimeoutRef.current = setTimeout(() => {
      setShowControls(false);
    }, 3000);
  };

  // Handle fullscreen
  const handleFullscreen = () => {
    const player = playerRef.current?.getInternalPlayer();
    if (player) {
      if (player.requestFullscreen) {
        player.requestFullscreen();
      } else if ((player as any).webkitRequestFullscreen) {
        (player as any).webkitRequestFullscreen();
      } else if ((player as any).mozRequestFullScreen) {
        (player as any).mozRequestFullScreen();
      } else if ((player as any).msRequestFullscreen) {
        (player as any).msRequestFullscreen();
      }
      setIsFullscreen(true);
    }
  };

  // Handle exit fullscreen
  const handleExitFullscreen = () => {
    if (document.exitFullscreen) {
      document.exitFullscreen();
    } else if ((document as any).webkitExitFullscreen) {
      (document as any).webkitExitFullscreen();
    } else if ((document as any).mozCancelFullScreen) {
      (document as any).mozCancelFullScreen();
    } else if ((document as any).msExitFullscreen) {
      (document as any).msExitFullscreen();
    }
    setIsFullscreen(false);
  };

  // Handle keyboard events
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (!playerRef.current) return;

      switch (e.key) {
        case ' ':
        case 'k':
          e.preventDefault();
          handlePlayPause();
          break;
        case 'ArrowLeft':
          e.preventDefault();
          playerRef.current.seekTo(playerRef.current.getCurrentTime() - 5);
          break;
        case 'ArrowRight':
          e.preventDefault();
          playerRef.current.seekTo(playerRef.current.getCurrentTime() + 5);
          break;
        case 'ArrowUp':
          e.preventDefault();
          handleVolumeChange(Math.min(volume + 0.1, 1));
          break;
        case 'ArrowDown':
          e.preventDefault();
          handleVolumeChange(Math.max(volume - 0.1, 0));
          break;
        case 'f':
          e.preventDefault();
          if (isFullscreen) {
            handleExitFullscreen();
          } else {
            handleFullscreen();
          }
          break;
        case 'm':
          e.preventDefault();
          handleMute();
          break;
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [playing, volume, isFullscreen]);

  // Clean up timeout on unmount
  useEffect(() => {
    return () => {
      if (controlsTimeoutRef.current) {
        clearTimeout(controlsTimeoutRef.current);
      }
    };
  }, []);

  const qualities = [
    { label: 'Auto', value: 'auto' },
    { label: '1080p', value: '1080p' },
    { label: '720p', value: '720p' },
    { label: '480p', value: '480p' },
    { label: '360p', value: '360p' },
  ];

  const playbackRates = [0.25, 0.5, 0.75, 1, 1.25, 1.5, 1.75, 2];

  return (
    <div className="relative bg-black rounded-lg overflow-hidden">
      {/* Video Player */}
      <div
        className="relative"
        onMouseMove={handleMouseMove}
        onMouseLeave={() => setShowControls(false)}
      >
        <ReactPlayer
          ref={playerRef}
          url={video.videoUrl}
          width="100%"
          height="100%"
          playing={playing}
          volume={volume}
          muted={muted}
          onProgress={handleProgress}
          onDuration={handleDuration}
          onReady={handleReady}
          playbackRate={playbackRate}
          controls={false}
          config={{
            file: {
              attributes: {
                crossOrigin: 'anonymous',
              },
            },
          }}
        />

        {/* Loading Overlay */}
        {!loaded && (
          <div className="absolute inset-0 flex items-center justify-center bg-black">
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-white"></div>
          </div>
        )}

        {/* Controls Overlay */}
        {showControls && (
          <div className="absolute inset-0 bg-gradient-to-t from-black/80 via-transparent to-transparent">
            {/* Top Bar */}
            <div className="absolute top-0 left-0 right-0 p-4 flex items-center justify-between">
              <button
                onClick={onBack}
                className="text-white hover:text-gray-300 transition-colors"
              >
                <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
                </svg>
              </button>
              <div className="flex items-center gap-2">
                <span className="text-white font-medium">{video.title}</span>
              </div>
              <div className="w-6"></div> {/* Spacer for alignment */}
            </div>

            {/* Center Play/Pause Button */}
            <div className="absolute inset-0 flex items-center justify-center">
              <button
                onClick={handlePlayPause}
                className="bg-white/20 backdrop-blur-sm rounded-full p-4 hover:bg-white/30 transition-colors"
              >
                {playing ? (
                  <Pause className="h-8 w-8 text-white" />
                ) : (
                  <Play className="h-8 w-8 text-white ml-1" />
                )}
              </button>
            </div>

            {/* Bottom Controls */}
            <div className="absolute bottom-0 left-0 right-0 p-4">
              {/* Progress Bar */}
              <div
                className="relative h-1 bg-gray-700 rounded-full mb-4 cursor-pointer"
                onClick={handleSeek}
              >
                <div
                  className="absolute top-0 left-0 h-full bg-red-600 rounded-full"
                  style={{ width: `${played * 100}%` }}
                />
                <div
                  className="absolute top-1/2 -translate-y-1/2 w-3 h-3 bg-white rounded-full shadow-lg"
                  style={{ left: `${played * 100}%` }}
                />
              </div>

              {/* Control Buttons */}
              <div className="flex items-center justify-between">
                <div className="flex items-center gap-4">
                  <button
                    onClick={handlePlayPause}
                    className="text-white hover:text-gray-300 transition-colors"
                  >
                    {playing ? <Pause className="h-6 w-6" /> : <Play className="h-6 w-6" />}
                  </button>
                  
                  <button
                    onClick={() => playerRef.current?.seekTo(playerRef.current.getCurrentTime() - 10)}
                    className="text-white hover:text-gray-300 transition-colors"
                  >
                    <SkipBack className="h-6 w-6" />
                  </button>
                  
                  <button
                    onClick={() => playerRef.current?.seekTo(playerRef.current.getCurrentTime() + 10)}
                    className="text-white hover:text-gray-300 transition-colors"
                  >
                    <SkipForward className="h-6 w-6" />
                  </button>

                  <div className="flex items-center gap-2">
                    <button
                      onClick={handleMute}
                      className="text-white hover:text-gray-300 transition-colors"
                    >
                      {muted || volume === 0 ? (
                        <VolumeX className="h-5 w-5" />
                      ) : (
                        <Volume2 className="h-5 w-5" />
                      )}
                    </button>
                    <input
                      type="range"
                      min={0}
                      max={1}
                      step={0.1}
                      value={muted ? 0 : volume}
                      onChange={(e) => handleVolumeChange(parseFloat(e.target.value))}
                      className="w-24 h-1 bg-gray-700 rounded-full appearance-none cursor-pointer"
                    />
                  </div>

                  <div className="text-white text-sm">
                    {formatDuration(played * duration)} / {formatDuration(duration)}
                  </div>
                </div>

                <div className="flex items-center gap-4">
                  <div className="flex items-center gap-2">
                    <select
                      value={quality}
                      onChange={(e) => setQuality(e.target.value)}
                      className="bg-gray-800 text-white text-sm rounded px-2 py-1"
                    >
                      {qualities.map((q) => (
                        <option key={q.value} value={q.value}>
                          {q.label}
                        </option>
                      ))}
                    </select>
                  </div>

                  <div className="flex items-center gap-2">
                    <select
                      value={playbackRate}
                      onChange={(e) => setPlaybackRate(parseFloat(e.target.value))}
                      className="bg-gray-800 text-white text-sm rounded px-2 py-1"
                    >
                      {playbackRates.map((rate) => (
                        <option key={rate} value={rate}>
                          {rate}x
                        </option>
                      ))}
                    </select>
                  </div>

                  <button
                    onClick={handleFullscreen}
                    className="text-white hover:text-gray-300 transition-colors"
                  >
                    <Fullscreen className="h-5 w-5" />
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Video Info */}
      <div className="p-4 bg-black/50 backdrop-blur-sm">
        <div className="flex items-start gap-4">
          <img
            src={video.creator.avatarUrl}
            alt={video.creator.username}
            className="w-10 h-10 rounded-full"
          />
          <div className="flex-1">
            <h3 className="text-white font-semibold text-lg mb-1">{video.title}</h3>
            <div className="flex items-center gap-4 text-gray-300 text-sm mb-2">
              <span>{formatViews(video.views)} views</span>
              <span>{formatDate(video.createdAt)}</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-gray-300 text-sm">
                {video.creator.username}
              </span>
              <button className="bg-red-600 hover:bg-red-700 text-white px-4 py-1 rounded-full text-sm transition-colors">
                Subscribe
              </button>
            </div>
          </div>
          <button className="text-gray-300 hover:text-white transition-colors">
            <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 12h.01M12 12h.01M19 12h.01M6 12a1 1 0 11-2 0 1 1 0 012 0zm7 0a1 1 0 11-2 0 1 1 0 012 0zm7 0a1 1 0 11-2 0 1 1 0 012 0z" />
            </svg>
          </button>
        </div>
        
        <div className="mt-4 text-gray-300 text-sm">
          {video.description}
        </div>

        <div className="flex items-center gap-6 mt-4">
          <button className="flex items-center gap-2 text-gray-300 hover:text-white transition-colors">
            <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M14 10h4.764a2 2 0 011.789 2.894l-3.5 7A2 2 0 0115.263 21h-4.017c-.163 0-.326-.02-.485-.06L7 20m7-10V5a2 2 0 00-2-2h-.095c-.5 0-.905.405-.905.905 0 .714-.211 1.412-.608 2.006L7 11v9m7-10h-2M7 20H5a2 2 0 01-2-2v-6a2 2 0 012-2h2.5" />
            </svg>
            <span>{formatViews(video.likes)}</span>
          </button>
          
          <button className="flex items-center gap-2 text-gray-300 hover:text-white transition-colors">
            <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
            </svg>
            <span>{video.comments}</span>
          </button>
          
          <button className="flex items-center gap-2 text-gray-300 hover:text-white transition-colors">
            <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8.684 13.342C8.886 12.938 9 12.482 9 12c0-.482-.114-.938-.316-1.342m0 2.684a3 3 0 110-2.684m0 2.684l6.632 3.316m-6.632-6l6.632-3.316m0 0a3 3 0 105.367-2.684 3 3 0 00-5.367 2.684zm0 9.316a3 3 0 105.368 2.684 3 3 0 00-5.368-2.684z" />
            </svg>
            <span>Share</span>
          </button>
          
          <button className="flex items-center gap-2 text-gray-300 hover:text-white transition-colors">
            <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 5a2 2 0 012-2h10a2 2 0 012 2v16l-7-3.5L5 21V5z" />
            </svg>
            <span>Save</span>
          </button>
        </div>
      </div>
    </div>
  );
}