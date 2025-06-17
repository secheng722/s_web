// 全局应用配置
const API_BASE_URL = '';

// 本地存储工具函数
const Storage = {
    get: (key) => {
        try {
            const item = localStorage.getItem(key);
            return item ? JSON.parse(item) : null;
        } catch (error) {
            console.error('Error reading from localStorage:', error);
            return null;
        }
    },
    
    set: (key, value) => {
        try {
            localStorage.setItem(key, JSON.stringify(value));
        } catch (error) {
            console.error('Error writing to localStorage:', error);
        }
    },
    
    remove: (key) => {
        try {
            localStorage.removeItem(key);
        } catch (error) {
            console.error('Error removing from localStorage:', error);
        }
    }
};

// HTTP 请求工具函数
const Http = {
    async request(url, options = {}) {
        const token = Storage.get('auth_token');
        
        const config = {
            method: 'GET',
            headers: {
                'Content-Type': 'application/json',
                ...options.headers
            },
            ...options
        };
        
        if (token) {
            config.headers['Authorization'] = `Bearer ${token}`;
        }
        
        try {
            const response = await fetch(API_BASE_URL + url, config);
            
            if (!response.ok) {
                const errorText = await response.text();
                throw new Error(`HTTP ${response.status}: ${errorText}`);
            }
            
            const contentType = response.headers.get('content-type');
            if (contentType && contentType.includes('application/json')) {
                return await response.json();
            } else {
                return await response.text();
            }
        } catch (error) {
            console.error('HTTP request failed:', error);
            throw error;
        }
    },
    
    get: (url, options = {}) => Http.request(url, { ...options, method: 'GET' }),
    
    post: (url, data, options = {}) => Http.request(url, {
        ...options,
        method: 'POST',
        body: JSON.stringify(data)
    }),
    
    put: (url, data, options = {}) => Http.request(url, {
        ...options,
        method: 'PUT',  
        body: JSON.stringify(data)
    }),
    
    delete: (url, options = {}) => Http.request(url, { ...options, method: 'DELETE' })
};

// 认证工具函数
const Auth = {
    isLoggedIn: () => !!Storage.get('auth_token'),
    
    getToken: () => Storage.get('auth_token'),
    
    login: (token) => {
        Storage.set('auth_token', token);
        updateAuthUI();
    },
    
    logout: () => {
        Storage.remove('auth_token');
        Storage.remove('user_info');
        updateAuthUI();
        window.location.href = '/';
    },
    
    getUser: () => Storage.get('user_info')
};

// 更新认证相关的 UI
function updateAuthUI() {
    const authLink = document.getElementById('auth-link');
    const logoutLink = document.getElementById('logout-link');
    const createLink = document.getElementById('create-link');
    
    if (Auth.isLoggedIn()) {
        if (authLink) {
            authLink.style.display = 'none';
        }
        if (logoutLink) {
            logoutLink.style.display = 'inline-block';
            logoutLink.onclick = (e) => {
                e.preventDefault();
                Auth.logout();
            };
        }
        if (createLink) {
            createLink.style.display = 'inline-block';
        }
    } else {
        if (authLink) {
            authLink.style.display = 'inline-block';
        }
        if (logoutLink) {
            logoutLink.style.display = 'none';
        }
        if (createLink) {
            createLink.style.display = 'none';
        }
    }
}

// 显示消息
function showMessage(element, message, type = 'error') {
    if (!element) return;
    
    element.textContent = message;
    element.className = `message ${type}`;
    element.style.display = 'block';
    
    // 3秒后自动隐藏成功消息
    if (type === 'success') {
        setTimeout(() => {
            element.style.display = 'none';
        }, 3000);
    }
}

// 格式化日期
function formatDate(dateString) {
    const date = new Date(dateString);
    return date.toLocaleDateString('zh-CN', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit'
    });
}

// 截取文本
function truncateText(text, maxLength = 150) {
    if (text.length <= maxLength) return text;
    return text.substring(0, maxLength) + '...';
}

// 页面加载完成后初始化
document.addEventListener('DOMContentLoaded', () => {
    updateAuthUI();
});
