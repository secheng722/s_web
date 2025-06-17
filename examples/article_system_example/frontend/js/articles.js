// 文章列表相关的 JavaScript

document.addEventListener('DOMContentLoaded', () => {
    // 加载文章列表
    loadArticles();
    
    // 刷新按钮
    const refreshBtn = document.getElementById('refresh-btn');
    if (refreshBtn) {
        refreshBtn.addEventListener('click', loadArticles);
    }
    
    // 更新认证状态
    updateAuthUI();
});

// 加载文章列表
async function loadArticles() {
    const loadingElement = document.getElementById('articles-loading');
    const containerElement = document.getElementById('articles-container');
    const errorElement = document.getElementById('articles-error');
    
    // 显示加载状态
    loadingElement.style.display = 'block';
    containerElement.innerHTML = '';
    errorElement.style.display = 'none';
    
    try {
        const articles = await Http.get('/api/articles');
        
        // 隐藏加载状态
        loadingElement.style.display = 'none';
        
        if (articles && articles.length > 0) {
            renderArticles(articles);
        } else {
            containerElement.innerHTML = `
                <div class="empty-state">
                    <h3>暂无文章</h3>
                    <p>还没有发布的文章，快来创建第一篇吧！</p>
                    ${Auth.isLoggedIn() ? '<a href="/create-article" class="btn btn-primary">创建文章</a>' : ''}
                </div>
            `;
        }
    } catch (error) {
        console.error('Failed to load articles:', error);
        loadingElement.style.display = 'none';
        errorElement.textContent = `加载文章失败：${error.message}`;
        errorElement.style.display = 'block';
    }
}

// 渲染文章列表
function renderArticles(articles) {
    const containerElement = document.getElementById('articles-container');
    
    const articlesHTML = articles.map(article => {
        const isOwner = Auth.isLoggedIn() && Auth.getUser() && Auth.getUser().id === article.author_id;
        
        return `
            <div class="article-card" data-id="${article.id}">
                <h3>${escapeHtml(article.title)}</h3>
                <div class="article-meta">
                    <span>作者：${escapeHtml(article.author_name || '未知')}</span>
                    <span>发布时间：${formatDate(article.created_at)}</span>
                    ${article.updated_at !== article.created_at ? `<span>更新时间：${formatDate(article.updated_at)}</span>` : ''}
                </div>
                <div class="article-excerpt">
                    ${escapeHtml(truncateText(article.content, 200))}
                </div>
                <div class="article-actions">
                    <button class="btn btn-secondary" onclick="viewArticle(${article.id})">查看详情</button>
                    ${isOwner ? `
                        <button class="btn btn-primary" onclick="editArticle(${article.id})">编辑</button>
                        <button class="btn btn-danger" onclick="deleteArticle(${article.id})">删除</button>
                    ` : ''}
                </div>
            </div>
        `;
    }).join('');
    
    containerElement.innerHTML = articlesHTML;
}

// 查看文章详情
function viewArticle(articleId) {
    // TODO: 实现文章详情页面
    alert(`查看文章 ${articleId} - 详情页面开发中`);
}

// 编辑文章
function editArticle(articleId) {
    // TODO: 实现文章编辑页面
    alert(`编辑文章 ${articleId} - 编辑页面开发中`);
}

// 删除文章
async function deleteArticle(articleId) {
    if (!confirm('确定要删除这篇文章吗？此操作不可恢复。')) {
        return;
    }
    
    try {
        await Http.delete(`/api/articles/${articleId}`);
        
        // 删除成功，重新加载文章列表
        loadArticles();
        
        // 显示成功消息
        const messageElement = document.createElement('div');
        messageElement.className = 'message success';
        messageElement.textContent = '文章删除成功';
        messageElement.style.position = 'fixed';
        messageElement.style.top = '20px';
        messageElement.style.right = '20px';
        messageElement.style.zIndex = '1000';
        
        document.body.appendChild(messageElement);
        
        setTimeout(() => {
            document.body.removeChild(messageElement);
        }, 3000);
        
    } catch (error) {
        console.error('Failed to delete article:', error);
        alert(`删除文章失败：${error.message}`);
    }
}

// HTML 转义函数
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// 空状态样式
const emptyStateStyle = `
    <style>
        .empty-state {
            text-align: center;
            padding: 4rem 2rem;
            color: #666;
            grid-column: 1 / -1;
        }
        
        .empty-state h3 {
            font-size: 1.5rem;
            margin-bottom: 1rem;
            color: #333;
        }
        
        .empty-state p {
            margin-bottom: 2rem;
            font-size: 1.1rem;
        }
        
        .btn-danger {
            background: #dc3545;
            color: white;
        }
        
        .btn-danger:hover {
            background: #c82333;
            transform: translateY(-2px);
        }
    </style>
`;

// 添加样式到页面
document.head.insertAdjacentHTML('beforeend', emptyStateStyle);
