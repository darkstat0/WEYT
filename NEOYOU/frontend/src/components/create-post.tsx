'use client';

import { useState, useRef } from 'react';
import { X, Upload, Video, Mic, Image, Music, MoreHorizontal } from 'lucide-react';

interface CreatePostProps {
  isOpen: boolean;
  onClose: () => void;
}

export function CreatePost({ isOpen, onClose }: CreatePostProps) {
  const [selectedTab, setSelectedTab] = useState<'video' | 'audio' | 'image' | 'stream'>('video');
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [isUploading, setIsUploading] = useState(false);
  const [uploadProgress, setUploadProgress] = useState(0);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (files && files.length > 0) {
      handleUpload(files[0]);
    }
  };

  const handleUpload = async (file: File) => {
    setIsUploading(true);
    setUploadProgress(0);

    // Simulate upload progress
    const interval = setInterval(() => {
      setUploadProgress((prev) => {
        if (prev >= 100) {
          clearInterval(interval);
          setIsUploading(false);
          return 100;
        }
        return prev + 10;
      });
    }, 200);

    // In a real app, you would upload the file to your server
    console.log('Uploading file:', file);
  };

  const handleDragOver = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
  };

  const handleDrop = (e: React.DragEvent<HTMLDivElement>) => {
    e.preventDefault();
    const files = e.dataTransfer.files;
    if (files && files.length > 0) {
      handleUpload(files[0]);
    }
  };

  const tabs = [
    { id: 'video', label: 'Video', icon: Video },
    { id: 'audio', label: 'Audio', icon: Mic },
    { id: 'image', label: 'Image', icon: Image },
    { id: 'stream', label: 'Go Live', icon: Music },
  ];

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div className="bg-white rounded-lg w-full max-w-2xl max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b">
          <h2 className="text-xl font-semibold">Create Post</h2>
          <button
            onClick={onClose}
            className="p-2 hover:bg-gray-100 rounded-full transition-colors"
          >
            <X className="h-5 w-5 text-gray-600" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-4">
          {/* Tabs */}
          <div className="flex gap-2 mb-6 border-b">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setSelectedTab(tab.id as any)}
                className={`flex items-center gap-2 px-4 py-2 font-medium transition-colors ${
                  selectedTab === tab.id
                    ? 'text-red-600 border-b-2 border-red-600'
                    : 'text-gray-500 hover:text-gray-700'
                }`}
              >
                <tab.icon className="h-4 w-4" />
                {tab.label}
              </button>
            ))}
          </div>

          {/* Upload Area */}
          <div
            className="border-2 border-dashed border-gray-300 rounded-lg p-8 text-center cursor-pointer hover:border-gray-400 transition-colors"
            onClick={() => fileInputRef.current?.click()}
            onDragOver={handleDragOver}
            onDrop={handleDrop}
          >
            <input
              ref={fileInputRef}
              type="file"
              className="hidden"
              accept={
                selectedTab === 'video'
                  ? 'video/*'
                  : selectedTab === 'audio'
                  ? 'audio/*'
                  : selectedTab === 'image'
                  ? 'image/*'
                  : 'video/*'
              }
              onChange={handleFileSelect}
            />

            {isUploading ? (
              <div className="flex flex-col items-center">
                <div className="w-16 h-16 border-4 border-gray-300 border-t-red-600 rounded-full animate-spin mb-4"></div>
                <p className="text-gray-600">Uploading... {uploadProgress}%</p>
              </div>
            ) : (
              <>
                <Upload className="h-12 w-12 text-gray-400 mx-auto mb-4" />
                <p className="text-lg font-medium text-gray-700 mb-2">
                  Drag & drop your {selectedTab} here
                </p>
                <p className="text-gray-500 mb-4">
                  or click to browse files
                </p>
                <p className="text-sm text-gray-400">
                  Maximum file size: 4GB
                </p>
              </>
            )}
          </div>

          {/* Title and Description */}
          <div className="mt-6 space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Title
              </label>
              <input
                type="text"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
                placeholder="Add a title"
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-red-500 focus:border-transparent"
              />
            </div>
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                Description
              </label>
              <textarea
                value={description}
                onChange={(e) => setDescription(e.target.value)}
                placeholder="Add a description"
                rows={3}
                className="w-full px-3 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-red-500 focus:border-transparent"
              />
            </div>
          </div>

          {/* Additional Options */}
          <div className="mt-6">
            <h3 className="text-sm font-medium text-gray-700 mb-3">Additional Options</h3>
            <div className="grid grid-cols-2 gap-4">
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="comments"
                  className="rounded border-gray-300 text-red-600 focus:ring-red-500"
                />
                <label htmlFor="comments" className="text-sm text-gray-600">
                  Allow comments
                </label>
              </div>
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="likes"
                  className="rounded border-gray-300 text-red-600 focus:ring-red-500"
                />
                <label htmlFor="likes" className="text-sm text-gray-600">
                  Allow likes
                </label>
              </div>
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="monetize"
                  className="rounded border-gray-300 text-red-600 focus:ring-red-500"
                />
                <label htmlFor="monetize" className="text-sm text-gray-600">
                  Monetize this content
                </label>
              </div>
              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="ageRestriction"
                  className="rounded border-gray-300 text-red-600 focus:ring-red-500"
                />
                <label htmlFor="ageRestriction" className="text-sm text-gray-600">
                  Age restriction
                </label>
              </div>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-between p-4 border-t bg-gray-50">
          <button className="flex items-center gap-2 px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg transition-colors">
            <MoreHorizontal className="h-5 w-5" />
            <span>More options</span>
          </button>
          <button
            onClick={() => {
              // Handle post creation
              console.log('Creating post with:', { title, description });
              onClose();
            }}
            className="px-6 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors disabled:opacity-50"
            disabled={!title || isUploading}
          >
            {isUploading ? 'Uploading...' : 'Post'}
          </button>
        </div>
      </div>
    </div>
  );
}