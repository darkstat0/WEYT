'use client';

import { useState } from 'react';
import { Search, Mic, Video, Bell, User, Menu, X } from 'lucide-react';

interface HeaderProps {
  onMenuToggle?: () => void;
  className?: string;
}

export function Header({ onMenuToggle, className = '' }: HeaderProps) {
  const [searchQuery, setSearchQuery] = useState('');
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    // Handle search logic here
    console.log('Searching for:', searchQuery);
  };

  const handleMenuToggle = () => {
    setIsMenuOpen(!isMenuOpen);
    onMenuToggle?.();
  };

  return (
    <header className={`${className} sticky top-0 z-50 bg-white border-b border-gray-200`}>
      <div className="flex items-center justify-between px-4 py-2">
        {/* Left Side */}
        <div className="flex items-center gap-4">
          {/* Menu Button (Mobile) */}
          <button
            onClick={handleMenuToggle}
            className="md:hidden p-2 rounded-lg hover:bg-gray-100 transition-colors"
          >
            {isMenuOpen ? <X className="h-5 w-5 text-gray-700" /> : <Menu className="h-5 w-5 text-gray-700" />}
          </button>

          {/* Logo */}
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 bg-red-600 rounded-lg flex items-center justify-center">
              <Video className="h-5 w-5 text-white" />
            </div>
            <span className="text-xl font-bold text-gray-900 hidden sm:inline">NeoYou</span>
          </div>
        </div>

        {/* Search Bar */}
        <div className="flex-1 max-w-2xl mx-4">
          <form onSubmit={handleSearchSubmit} className="relative">
            {/* Search Input */}
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search"
              className="w-full px-4 py-2 pl-10 pr-4 border border-gray-300 rounded-full focus:outline-none focus:border-gray-400"
            />

            {/* Search Icon */}
            <div className="absolute left-3 top-1/2 -translate-y-1/2">
              <Search className="h-5 w-5 text-gray-400" />
            </div>

            {/* Voice Search Button */}
            <button
              type="button"
              className="absolute right-2 top-1/2 -translate-y-1/2 p-1 rounded-full hover:bg-gray-100 transition-colors"
            >
              <Mic className="h-5 w-5 text-gray-500" />
            </button>
          </form>
        </div>

        {/* Right Side */}
        <div className="flex items-center gap-2">
          {/* Create Button */}
          <button className="flex items-center gap-1 px-4 py-2 bg-red-600 text-white rounded-full hover:bg-red-700 transition-colors">
            <Video className="h-5 w-5" />
            <span className="hidden sm:inline">Create</span>
          </button>

          {/* Notifications */}
          <button className="p-2 rounded-lg hover:bg-gray-100 transition-colors relative">
            <Bell className="h-5 w-5 text-gray-700" />
            <span className="absolute top-1 right-1 w-2 h-2 bg-red-600 rounded-full"></span>
          </button>

          {/* User Avatar */}
          <button className="flex items-center gap-2 p-2 rounded-lg hover:bg-gray-100 transition-colors">
            <div className="w-8 h-8 bg-gray-300 rounded-full flex items-center justify-center">
              <User className="h-4 w-4 text-gray-600" />
            </div>
            <span className="hidden sm:inline text-sm font-medium text-gray-700">Account</span>
          </button>
        </div>
      </div>

      {/* Mobile Search Bar (Below Header) */}
      {isMenuOpen && (
        <div className="md:hidden border-t border-gray-200 p-4 bg-white">
          <form onSubmit={handleSearchSubmit} className="relative">
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              placeholder="Search"
              className="w-full px-4 py-2 pl-10 pr-4 border border-gray-300 rounded-full focus:outline-none focus:border-gray-400"
            />
            <div className="absolute left-3 top-1/2 -translate-y-1/2">
              <Search className="h-5 w-5 text-gray-400" />
            </div>
          </form>
        </div>
      )}
    </header>
  );
}