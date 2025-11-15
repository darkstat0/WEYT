'use client';

import { Home, TrendingUp, PlayCircle, Clock, Heart, Star, User, Settings, Bell, HelpCircle } from 'lucide-react';

interface SidebarProps {
  className?: string;
}

export function Sidebar({ className = '' }: SidebarProps) {
  const menuItems = [
    { id: 'home', label: 'Home', icon: Home, active: true },
    { id: 'trending', label: 'Trending', icon: TrendingUp },
    { id: 'subscriptions', label: 'Subscriptions', icon: PlayCircle },
    { id: 'history', label: 'History', icon: Clock },
    { id: 'liked', label: 'Liked videos', icon: Heart },
    { id: 'premium', label: 'Premium', icon: Star, premium: true },
  ];

  const otherItems = [
    { id: 'channels', label: 'Channels', icon: User },
    { id: 'settings', label: 'Settings', icon: Settings },
    { id: 'notifications', label: 'Notifications', icon: Bell },
    { id: 'help', label: 'Help & feedback', icon: HelpCircle },
  ];

  return (
    <div className={`${className} w-64 flex-shrink-0`}>
      {/* Logo */}
      <div className="flex items-center gap-2 px-4 py-3 mb-6">
        <div className="w-8 h-8 bg-red-600 rounded-lg flex items-center justify-center">
          <PlayCircle className="h-5 w-5 text-white" />
        </div>
        <span className="text-xl font-bold text-white">NeoYou</span>
      </div>

      {/* Main Menu */}
      <nav className="mb-8">
        <ul className="space-y-1">
          {menuItems.map((item) => (
            <li key={item.id}>
              <a
                href={`#${item.id}`}
                className={`flex items-center gap-3 px-4 py-2.5 rounded-lg transition-colors ${
                  item.active
                    ? 'bg-white/10 text-white'
                    : 'text-gray-300 hover:bg-gray-800 hover:text-white'
                }`}
              >
                <item.icon className="h-5 w-5" />
                <span className="font-medium">{item.label}</span>
                {item.premium && (
                  <span className="ml-auto bg-yellow-500 text-black text-xs px-1.5 py-0.5 rounded-full">
                    Premium
                  </span>
                )}
              </a>
            </li>
          ))}
        </ul>
      </nav>

      {/* Other Links */}
      <nav className="mb-8">
        <ul className="space-y-1">
          {otherItems.map((item) => (
            <li key={item.id}>
              <a
                href={`#${item.id}`}
                className="flex items-center gap-3 px-4 py-2.5 rounded-lg text-gray-300 hover:bg-gray-800 hover:text-white transition-colors"
              >
                <item.icon className="h-5 w-5" />
                <span className="font-medium">{item.label}</span>
              </a>
            </li>
          ))}
        </ul>
      </nav>

      {/* Divider */}
      <div className="border-t border-gray-800 my-6"></div>

      {/* Quick Access */}
      <div className="px-4 mb-6">
        <h3 className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-3">
          Quick Access
        </h3>
        <ul className="space-y-2">
          <li>
            <a
              href="#"
              className="flex items-center gap-3 px-3 py-1.5 rounded-lg text-gray-300 hover:bg-gray-800 hover:text-white transition-colors"
            >
              <span className="text-sm">Recently watched</span>
            </a>
          </li>
          <li>
            <a
              href="#"
              className="flex items-center gap-3 px-3 py-1.5 rounded-lg text-gray-300 hover:bg-gray-800 hover:text-white transition-colors"
            >
              <span className="text-sm">Watch later</span>
            </a>
          </li>
          <li>
            <a
              href="#"
              className="flex items-center gap-3 px-3 py-1.5 rounded-lg text-gray-300 hover:bg-gray-800 hover:text-white transition-colors"
            >
              <span className="text-sm">Your videos</span>
            </a>
          </li>
        </ul>
      </div>

      {/* Divider */}
      <div className="border-t border-gray-800 my-6"></div>

      {/* Categories */}
      <div className="px-4">
        <h3 className="text-xs font-semibold text-gray-400 uppercase tracking-wider mb-3">
          Categories
        </h3>
        <ul className="space-y-2">
          <li>
            <a
              href="#"
              className="flex items-center gap-3 px-3 py-1.5 rounded-lg text-gray-300 hover:bg-gray-800 hover:text-white transition-colors"
            >
              <span className="text-sm">Gaming</span>
            </a>
          </li>
          <li>
            <a
              href="#"
              className="flex items-center gap-3 px-3 py-1.5 rounded-lg text-gray-300 hover:bg-gray-800 hover:text-white transition-colors"
            >
              <span className="text-sm">Music</span>
            </a>
          </li>
          <li>
            <a
              href="#"
              className="flex items-center gap-3 px-3 py-1.5 rounded-lg text-gray-300 hover:bg-gray-800 hover:text-white transition-colors"
            >
              <span className="text-sm">Live</span>
            </a>
          </li>
          <li>
            <a
              href="#"
              className="flex items-center gap-3 px-3 py-1.5 rounded-lg text-gray-300 hover:bg-gray-800 hover:text-white transition-colors"
            >
              <span className="text-sm">News</span>
            </a>
          </li>
          <li>
            <a
              href="#"
              className="flex items-center gap-3 px-3 py-1.5 rounded-lg text-gray-300 hover:bg-gray-800 hover:text-white transition-colors"
            >
              <span className="text-sm">Sports</span>
            </a>
          </li>
        </ul>
      </div>

      {/* Premium Banner */}
      <div className="mt-8 px-4">
        <div className="bg-gradient-to-r from-red-600 to-red-700 rounded-lg p-4">
          <div className="flex items-center gap-3 mb-2">
            <Star className="h-5 w-5 text-yellow-400" />
            <h3 className="text-white font-semibold">Go Premium</h3>
          </div>
          <p className="text-red-100 text-sm mb-3">
            Enjoy exclusive videos, ad-free experience, and more
          </p>
          <button className="w-full bg-white text-red-600 font-medium py-1.5 rounded-lg hover:bg-gray-100 transition-colors">
            Upgrade Now
          </button>
        </div>
      </div>

      {/* Footer */}
      <div className="mt-8 px-4 text-xs text-gray-500">
        <p>© 2024 NeoYou</p>
        <p className="mt-1">Terms • Privacy • Safety</p>
      </div>
    </div>
  );
}