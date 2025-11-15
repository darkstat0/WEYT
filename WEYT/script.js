// Mobile menu toggle
const menuIcon = document.querySelector('.menu-icon');
const sidebar = document.querySelector('.sidebar');

menuIcon.addEventListener('click', () => {
    sidebar.classList.toggle('open');
});

// Close sidebar when clicking outside
document.addEventListener('click', (e) => {
    if (!sidebar.contains(e.target) && !menuIcon.contains(e.target)) {
        sidebar.classList.remove('open');
    }
});

// Search functionality
const searchInput = document.querySelector('.search-input');
const searchButton = document.querySelector('.search-button');

searchButton.addEventListener('click', () => {
    performSearch();
});

searchInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
        performSearch();
    }
});

function performSearch() {
    const query = searchInput.value.trim();
    if (query) {
        // In a real application, this would redirect to search results
        console.log('Searching for:', query);
        // You could implement actual search functionality here
        alert(`Searching for: ${query}`);
    }
}

// Category filtering
const categoryItems = document.querySelectorAll('.category-item');

categoryItems.forEach(item => {
    item.addEventListener('click', (e) => {
        e.preventDefault();
        
        // Remove active class from all items
        categoryItems.forEach(cat => cat.classList.remove('active'));
        
        // Add active class to clicked item
        item.classList.add('active');
        
        const category = item.textContent;
        filterVideosByCategory(category);
    });
});

function filterVideosByCategory(category) {
    console.log(`Filtering videos by category: ${category}`);
    // In a real application, this would filter the video grid
    // For now, we'll just log the action
}

// Video click handling
const videoCards = document.querySelectorAll('.video-card');

videoCards.forEach(card => {
    card.addEventListener('click', () => {
        const videoTitle = card.querySelector('.video-title').textContent;
        const channelName = card.querySelector('.video-channel').textContent;
        console.log(`Clicked video: "${videoTitle}" by ${channelName}`);
        
        // In a real application, this would navigate to the video page
        // For now, we'll show a simple notification
        showVideoNotification(videoTitle, channelName);
    });
});

function showVideoNotification(title, channel) {
    // Create notification element
    const notification = document.createElement('div');
    notification.className = 'video-notification';
    notification.innerHTML = `
        <div class="notification-content">
            <h4>Video Selected</h4>
            <p>${title}</p>
            <small>by ${channel}</small>
        </div>
    `;
    
    // Add notification styles
    notification.style.cssText = `
        position: fixed;
        top: 70px;
        right: 20px;
        background: #030303;
        color: white;
        padding: 16px 20px;
        border-radius: 8px;
        box-shadow: 0 4px 12px rgba(0,0,0,0.3);
        z-index: 1001;
        max-width: 300px;
        transform: translateX(100%);
        transition: transform 0.3s ease;
    `;
    
    document.body.appendChild(notification);
    
    // Animate in
    setTimeout(() => {
        notification.style.transform = 'translateX(0)';
    }, 100);
    
    // Remove after 3 seconds
    setTimeout(() => {
        notification.style.transform = 'translateX(100%)';
        setTimeout(() => {
            document.body.removeChild(notification);
        }, 300);
    }, 3000);
}

// Like/unlike functionality simulation
document.addEventListener('click', (e) => {
    if (e.target.textContent === '👍' || e.target.textContent === '👎') {
        e.stopPropagation();
        toggleLikeDislike(e.target);
    }
});

function toggleLikeDislike(button) {
    const isLike = button.textContent === '👍';
    const currentState = button.dataset.state || 'neutral';
    
    if (currentState === 'active') {
        // Remove active state
        button.dataset.state = 'neutral';
        button.style.opacity = '0.5';
    } else {
        // Set active state
        button.dataset.state = 'active';
        button.style.opacity = '1';
        
        // If opposite button was active, deactivate it
        const oppositeButton = isLike ? button.nextElementSibling : button.previousElementSibling;
        if (oppositeButton && oppositeButton.dataset.state === 'active') {
            oppositeButton.dataset.state = 'neutral';
            oppositeButton.style.opacity = '0.5';
        }
    }
}

// Add like/dislike buttons to video cards (this would normally be server-rendered)
function addLikeDislikeButtons() {
    const videoInfos = document.querySelectorAll('.video-info');
    
    videoInfos.forEach(info => {
        const buttonContainer = document.createElement('div');
        buttonContainer.className = 'video-actions';
        buttonContainer.style.cssText = `
            display: flex;
            gap: 8px;
            margin-top: 8px;
        `;
        
        const likeButton = document.createElement('button');
        likeButton.textContent = '👍';
        likeButton.style.cssText = `
            background: none;
            border: none;
            font-size: 14px;
            cursor: pointer;
            opacity: 0.5;
            transition: opacity 0.2s;
        `;
        
        const dislikeButton = document.createElement('button');
        dislikeButton.textContent = '👎';
        dislikeButton.style.cssText = `
            background: none;
            border: none;
            font-size: 14px;
            cursor: pointer;
            opacity: 0.5;
            transition: opacity 0.2s;
        `;
        
        buttonContainer.appendChild(likeButton);
        buttonContainer.appendChild(dislikeButton);
        info.appendChild(buttonContainer);
    });
}

// Initialize like/dislike buttons
addLikeDislikeButtons();

// Notification system
function showNotification(message, type = 'info') {
    const notification = document.createElement('div');
    notification.className = `notification ${type}`;
    notification.textContent = message;
    
    notification.style.cssText = `
        position: fixed;
        top: 70px;
        right: 20px;
        background: ${type === 'success' ? '#4caf50' : type === 'error' ? '#f44336' : '#2196f3'};
        color: white;
        padding: 12px 16px;
        border-radius: 4px;
        box-shadow: 0 2px 8px rgba(0,0,0,0.2);
        z-index: 1001;
        transform: translateX(100%);
        transition: transform 0.3s ease;
    `;
    
    document.body.appendChild(notification);
    
    setTimeout(() => {
        notification.style.transform = 'translateX(0)';
    }, 100);
    
    setTimeout(() => {
        notification.style.transform = 'translateX(100%)';
        setTimeout(() => {
            document.body.removeChild(notification);
        }, 300);
    }, 3000);
}

// Keyboard shortcuts
document.addEventListener('keydown', (e) => {
    // Ctrl/Cmd + K to focus search
    if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault();
        searchInput.focus();
    }
    
    // Escape to close sidebar
    if (e.key === 'Escape') {
        sidebar.classList.remove('open');
    }
    
    // / to focus search
    if (e.key === '/' && document.activeElement !== searchInput) {
        e.preventDefault();
        searchInput.focus();
    }
});

// Simulate real-time updates (like view count changes)
function simulateViewCountUpdates() {
    const videoStats = document.querySelectorAll('.video-stats');
    
    setInterval(() => {
        const randomIndex = Math.floor(Math.random() * videoStats.length);
        const statElement = videoStats[randomIndex];
        const currentText = statElement.textContent;
        
        // Extract current view count
        const viewMatch = currentText.match(/(\d+(?:\.\d+)?[KMB]?)\s+views/);
        if (viewMatch) {
            const currentViews = viewMatch[1];
            let newViews;
            
            if (currentViews.includes('K')) {
                const num = parseFloat(currentViews);
                newViews = `${(num + 0.1).toFixed(1)}K`;
            } else if (currentViews.includes('M')) {
                const num = parseFloat(currentViews);
                newViews = `${(num + 0.01).toFixed(2)}M`;
            } else {
                const num = parseInt(currentViews);
                newViews = (num + 1).toLocaleString();
            }
            
            const timeAgo = statElement.textContent.match(/\d+\s+(seconds?|minutes?|hours?|days?)\s+ago/);
            const timeText = timeAgo ? timeAgo[0] : 'just now';
            
            statElement.textContent = `${newViews} views • ${timeText}`;
        }
    }, 5000); // Update every 5 seconds
}

// Start simulation
simulateViewCountUpdates();

// Smooth scrolling for better UX
document.querySelectorAll('a[href^="#"]').forEach(anchor => {
    anchor.addEventListener('click', function (e) {
        e.preventDefault();
        const target = document.querySelector(this.getAttribute('href'));
        if (target) {
            target.scrollIntoView({
                behavior: 'smooth'
            });
        }
    });
});

// Add loading animation for videos
function addLoadingAnimation() {
    const videoCards = document.querySelectorAll('.video-card');
    
    videoCards.forEach((card, index) => {
        card.style.opacity = '0';
        card.style.transform = 'translateY(20px)';
        
        setTimeout(() => {
            card.style.transition = 'all 0.5s ease';
            card.style.opacity = '1';
            card.style.transform = 'translateY(0)';
        }, index * 100);
    });
}

// Initialize animations when page loads
window.addEventListener('load', () => {
    addLoadingAnimation();
});

// Performance optimization: Lazy load images
if ('IntersectionObserver' in window) {
    const imageObserver = new IntersectionObserver((entries, observer) => {
        entries.forEach(entry => {
            if (entry.isIntersecting) {
                const img = entry.target;
                img.style.opacity = '0';
                img.addEventListener('load', () => {
                    img.style.transition = 'opacity 0.3s ease';
                    img.style.opacity = '1';
                });
                observer.unobserve(img);
            }
        });
    });
    
    document.querySelectorAll('.video-thumbnail img').forEach(img => {
        imageObserver.observe(img);
    });
}