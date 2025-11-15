'use client';

import { useState } from 'react';
import { 
  Users, 
  Video, 
  Shield, 
  DollarSign, 
  BarChart3, 
  Settings, 
  Bell, 
  Activity,
  Database,
  Server,
  Wifi,
  HardDrive,
  Cpu,
  MemoryStick,
  AlertTriangle,
  CheckCircle,
  XCircle
} from 'lucide-react';

interface AdminPanelProps {
  isOpen: boolean;
  onClose: () => void;
}

export function AdminPanel({ isOpen, onClose }: AdminPanelProps) {
  const [activeTab, setActiveTab] = useState('dashboard');
  const [selectedUser, setSelectedUser] = useState<string | null>(null);
  const [selectedVideo, setSelectedVideo] = useState<string | null>(null);

  // Mock data for dashboard
  const dashboardStats = {
    totalUsers: 1250000,
    activeUsers: 450000,
    totalVideos: 8500000,
    totalViews: 4500000000,
    revenue: 1250000,
    reports: 450,
    flaggedContent: 120,
    systemHealth: 98
  };

  // Mock data for recent activity
  const recentActivity = [
    { id: '1', type: 'user', action: 'New user registered', time: '2 minutes ago', user: 'john_doe' },
    { id: '2', type: 'video', action: 'Video uploaded', time: '5 minutes ago', user: 'jane_smith' },
    { id: '3', type: 'moderation', action: 'Content flagged', time: '10 minutes ago', user: 'moderator_bot' },
    { id: '4', type: 'system', action: 'Server maintenance', time: '15 minutes ago', user: 'system_admin' },
    { id: '5', type: 'revenue', action: 'Payment processed', time: '20 minutes ago', user: 'payment_gateway' }
  ];

  // Mock data for system metrics
  const systemMetrics = {
    cpu: 45,
    memory: 62,
    disk: 78,
    network: 34,
    database: 55,
    cache: 28
  };

  // Mock data for alerts
  const alerts = [
    { id: '1', type: 'warning', message: 'High CPU usage detected', service: 'Backend API', time: '5 minutes ago' },
    { id: '2', type: 'error', message: 'Database connection timeout', service: 'PostgreSQL', time: '10 minutes ago' },
    { id: '3', type: 'info', message: 'SSL certificate expires in 30 days', service: 'Nginx', time: '1 hour ago' }
  ];

  // Mock data for recent reports
  const recentReports = [
    { id: '1', user: 'user123', video: 'video456', reason: 'Copyright violation', status: 'pending', time: '2 hours ago' },
    { id: '2', user: 'user789', video: 'video012', reason: 'Inappropriate content', status: 'investigating', time: '3 hours ago' },
    { id: '3', user: 'user456', video: 'video789', reason: 'Spam', status: 'resolved', time: '5 hours ago' }
  ];

  // Mock data for revenue stats
  const revenueStats = {
    today: 12500,
    week: 87500,
    month: 350000,
    year: 1250000,
    topEarners: [
      { creator: 'TechGuru', earnings: 45000 },
      { creator: 'MusicMaster', earnings: 32000 },
      { creator: 'GamerPro', earnings: 28000 },
      { creator: 'ChefLife', earnings: 25000 },
      { creator: 'TravelVibes', earnings: 20000 }
    ]
  };

  // Mock data for recent users
  const recentUsers = [
    { id: '1', username: 'new_user_123', email: 'user@example.com', role: 'user', registered: '2 hours ago' },
    { id: '2', username: 'creator_pro', email: 'creator@example.com', role: 'creator', registered: '5 hours ago' },
    { id: '3', username: 'premium_user', email: 'premium@example.com', role: 'premium', registered: '1 day ago' }
  ];

  const tabs = [
    { id: 'dashboard', label: 'Dashboard', icon: BarChart3 },
    { id: 'users', label: 'Users', icon: Users },
    { id: 'videos', label: 'Videos', icon: Video },
    { id: 'moderation', label: 'Moderation', icon: Shield },
    { id: 'revenue', label: 'Revenue', icon: DollarSign },
    { id: 'settings', label: 'Settings', icon: Settings },
    { id: 'logs', label: 'Logs', icon: Activity }
  ];

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div className="bg-white rounded-lg w-full max-w-6xl max-h-[90vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b">
          <h2 className="text-xl font-semibold">Admin Panel</h2>
          <button
            onClick={onClose}
            className="p-2 hover:bg-gray-100 rounded-full transition-colors"
          >
            <XCircle className="h-5 w-5 text-gray-600" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto flex">
          {/* Sidebar */}
          <div className="w-64 border-r p-4">
            <nav className="space-y-1">
              {tabs.map((tab) => (
                <button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id)}
                  className={`flex items-center gap-3 w-full px-3 py-2 rounded-lg text-left transition-colors ${
                    activeTab === tab.id
                      ? 'bg-red-600 text-white'
                      : 'text-gray-700 hover:bg-gray-100'
                  }`}
                >
                  <tab.icon className="h-5 w-5" />
                  <span className="font-medium">{tab.label}</span>
                </button>
              ))}
            </nav>

            {/* System Status */}
            <div className="mt-6 p-3 bg-green-50 rounded-lg border border-green-200">
              <div className="flex items-center gap-2 mb-2">
                <CheckCircle className="h-4 w-4 text-green-600" />
                <span className="text-sm font-medium text-green-800">System Status</span>
              </div>
              <div className="text-xs text-green-700">
                <p>All systems operational</p>
                <p className="mt-1">Last checked: 2 minutes ago</p>
              </div>
            </div>
          </div>

          {/* Main Content */}
          <div className="flex-1 p-6 overflow-y-auto">
            {/* Dashboard Tab */}
            {activeTab === 'dashboard' && (
              <div className="space-y-6">
                {/* Stats Grid */}
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                  <div className="bg-blue-50 p-4 rounded-lg border border-blue-200">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm text-blue-600">Total Users</p>
                        <p className="text-2xl font-bold text-blue-800">
                          {(dashboardStats.totalUsers / 1000000).toFixed(1)}M
                        </p>
                      </div>
                      <Users className="h-8 w-8 text-blue-600" />
                    </div>
                  </div>
                  
                  <div className="bg-green-50 p-4 rounded-lg border border-green-200">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm text-green-600">Active Users</p>
                        <p className="text-2xl font-bold text-green-800">
                          {(dashboardStats.activeUsers / 1000000).toFixed(1)}M
                        </p>
                      </div>
                      <Activity className="h-8 w-8 text-green-600" />
                    </div>
                  </div>
                  
                  <div className="bg-purple-50 p-4 rounded-lg border border-purple-200">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm text-purple-600">Total Videos</p>
                        <p className="text-2xl font-bold text-purple-800">
                          {(dashboardStats.totalVideos / 1000000).toFixed(1)}M
                        </p>
                      </div>
                      <Video className="h-8 w-8 text-purple-600" />
                    </div>
                  </div>
                  
                  <div className="bg-yellow-50 p-4 rounded-lg border border-yellow-200">
                    <div className="flex items-center justify-between">
                      <div>
                        <p className="text-sm text-yellow-600">Revenue</p>
                        <p className="text-2xl font-bold text-yellow-800">
                          ${dashboardStats.revenue.toLocaleString()}
                        </p>
                      </div>
                      <DollarSign className="h-8 w-8 text-yellow-600" />
                    </div>
                  </div>
                </div>

                {/* System Metrics */}
                <div className="bg-gray-50 p-4 rounded-lg border border-gray-200">
                  <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
                    <Server className="h-5 w-5" />
                    System Metrics
                  </h3>
                  <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div>
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-sm font-medium">CPU Usage</span>
                        <span className="text-sm text-gray-600">{systemMetrics.cpu}%</span>
                      </div>
                      <div className="w-full bg-gray-200 rounded-full h-2">
                        <div 
                          className="bg-blue-600 h-2 rounded-full" 
                          style={{ width: `${systemMetrics.cpu}%` }}
                        ></div>
                      </div>
                    </div>
                    
                    <div>
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-sm font-medium">Memory Usage</span>
                        <span className="text-sm text-gray-600">{systemMetrics.memory}%</span>
                      </div>
                      <div className="w-full bg-gray-200 rounded-full h-2">
                        <div 
                          className="bg-green-600 h-2 rounded-full" 
                          style={{ width: `${systemMetrics.memory}%` }}
                        ></div>
                      </div>
                    </div>
                    
                    <div>
                      <div className="flex items-center justify-between mb-2">
                        <span className="text-sm font-medium">Disk Usage</span>
                        <span className="text-sm text-gray-600">{systemMetrics.disk}%</span>
                      </div>
                      <div className="w-full bg-gray-200 rounded-full h-2">
                        <div 
                          className="bg-yellow-600 h-2 rounded-full" 
                          style={{ width: `${systemMetrics.disk}%` }}
                        ></div>
                      </div>
                    </div>
                  </div>
                </div>

                {/* Alerts */}
                <div className="bg-gray-50 p-4 rounded-lg border border-gray-200">
                  <h3 className="text-lg font-semibold mb-4 flex items-center gap-2">
                    <Bell className="h-5 w-5" />
                    Recent Alerts
                  </h3>
                  <div className="space-y-3">
                    {alerts.map((alert) => (
                      <div key={alert.id} className="flex items-start gap-3 p-3 bg-white rounded border">
                        {alert.type === 'warning' && <AlertTriangle className="h-5 w-5 text-yellow-600 mt-0.5" />}
                        {alert.type === 'error' && <XCircle className="h-5 w-5 text-red-600 mt-0.5" />}
                        {alert.type === 'info' && <CheckCircle className="h-5 w-5 text-blue-600 mt-0.5" />}
                        <div className="flex-1">
                          <p className="text-sm font-medium">{alert.message}</p>
                          <p className="text-xs text-gray-600">
                            {alert.service} • {alert.time}
                          </p>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            )}

            {/* Users Tab */}
            {activeTab === 'users' && (
              <div className="space-y-6">
                <div className="flex items-center justify-between">
                  <h3 className="text-lg font-semibold">User Management</h3>
                  <button className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors">
                    Add User
                  </button>
                </div>

                {/* Search and Filters */}
                <div className="flex gap-4">
                  <input
                    type="text"
                    placeholder="Search users..."
                    className="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-red-500"
                  />
                  <select className="px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-red-500">
                    <option>All Roles</option>
                    <option>User</option>
                    <option>Creator</option>
                    <option>Premium</option>
                    <option>Admin</option>
                  </select>
                </div>

                {/* Users Table */}
                <div className="bg-white rounded-lg border overflow-hidden">
                  <table className="w-full">
                    <thead className="bg-gray-50">
                      <tr>
                        <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">User</th>
                        <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Email</th>
                        <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Role</th>
                        <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Status</th>
                        <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Joined</th>
                        <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-gray-200">
                      {recentUsers.map((user) => (
                        <tr key={user.id} className="hover:bg-gray-50">
                          <td className="px-4 py-3 whitespace-nowrap">
                            <div className="flex items-center gap-3">
                              <div className="w-8 h-8 bg-gray-300 rounded-full flex items-center justify-center">
                                <span className="text-sm font-medium text-gray-700">
                                  {user.username.charAt(0).toUpperCase()}
                                </span>
                              </div>
                              <span className="text-sm font-medium text-gray-900">{user.username}</span>
                            </div>
                          </td>
                          <td className="px-4 py-3 whitespace-nowrap text-sm text-gray-500">{user.email}</td>
                          <td className="px-4 py-3 whitespace-nowrap">
                            <span className="px-2 py-1 text-xs rounded-full bg-blue-100 text-blue-800">
                              {user.role}
                            </span>
                          </td>
                          <td className="px-4 py-3 whitespace-nowrap">
                            <span className="px-2 py-1 text-xs rounded-full bg-green-100 text-green-800">
                              Active
                            </span>
                          </td>
                          <td className="px-4 py-3 whitespace-nowrap text-sm text-gray-500">{user.registered}</td>
                          <td className="px-4 py-3 whitespace-nowrap text-sm font-medium">
                            <button className="text-red-600 hover:text-red-900 mr-3">Edit</button>
                            <button className="text-red-600 hover:text-red-900">Ban</button>
                          </td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              </div>
            )}

            {/* Videos Tab */}
            {activeTab === 'videos' && (
              <div className="space-y-6">
                <div className="flex items-center justify-between">
                  <h3 className="text-lg font-semibold">Video Management</h3>
                  <button className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors">
                    Upload Video
                  </button>
                </div>

                {/* Video Stats */}
                <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                  <div className="bg-blue-50 p-4 rounded-lg border border-blue-200">
                    <p className="text-sm text-blue-600">Total Videos</p>
                    <p className="text-2xl font-bold text-blue-800">
                      {(dashboardStats.totalVideos / 1000000).toFixed(1)}M
                    </p>
                  </div>
                  
                  <div className="bg-green-50 p-4 rounded-lg border border-green-200">
                    <p className="text-sm text-green-600">Total Views</p>
                    <p className="text-2xl font-bold text-green-800">
                      {(dashboardStats.totalViews / 1000000000).toFixed(1)}B
                    </p>
                  </div>
                  
                  <div className="bg-purple-50 p-4 rounded-lg border border-purple-200">
                    <p className="text-sm text-purple-600">Processing</p>
                    <p className="text-2xl font-bold text-purple-800">234</p>
                  </div>
                </div>

                {/* Recent Videos */}
                <div className="bg-white rounded-lg border overflow-hidden">
                  <table className="w-full">
                    <thead className="bg-gray-50">
                      <tr>
                        <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Video</th>
                        <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Creator</th>
                        <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Views</th>
                        <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Status</th>
                        <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Uploaded</th>
                        <th className="px-4 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>
                      </tr>
                    </thead>
                    <tbody className="divide-y divide-gray-200">
                      {/* Video rows would go here */}
                    </tbody>
                  </table>
                </div>
              </div>
            )}

            {/* Other tabs would be implemented similarly */}
            {['moderation', 'revenue', 'settings', 'logs'].includes(activeTab) && (
              <div className="text-center py-12">
                <p className="text-gray-500">Content for {activeTab} tab coming soon...</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}