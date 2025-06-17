// 创建文章相关的 JavaScript

document.addEventListener('DOMContentLoaded', () => {
    // 检查登录状态
    if (!Auth.isLoggedIn()) {
        alert('请先登录');
        window.location.href = '/login';
        return;
    }
    
    // 创建文章表单处理
    const createForm = document.getElementById('create-article-form');
    if (createForm) {
        createForm.addEventListener('submit', handleCreateArticle);
    }
    
    // 更新认证状态
    updateAuthUI();
});

// 处理创建文章
async function handleCreateArticle(event) {
    event.preventDefault();
    
    const form = event.target;
    const formData = new FormData(form);
    const articleData = {
        title: formData.get('title').trim(),
        content: formData.get('content').trim()
    };
    
    const messageElement = document.getElementById('create-message');
    const submitButton = form.querySelector('button[type="submit"]');
    
    // 验证输入
    if (!articleData.title) {
        showMessage(messageElement, '请输入文章标题', 'error');
        return;
    }
    
    if (!articleData.content) {
        showMessage(messageElement, '请输入文章内容', 'error');
        return;
    }
    
    if (articleData.title.length > 200) {
        showMessage(messageElement, '文章标题不能超过200个字符', 'error');
        return;
    }
    
    try {
        // 禁用提交按钮
        submitButton.disabled = true;
        submitButton.textContent = '发布中...';
        
        // 发送创建请求
        const response = await Http.post('/api/articles/protected', articleData);
        
        showMessage(messageElement, '文章发布成功！正在跳转...', 'success');
        
        // 清空表单
        form.reset();
        
        // 跳转到文章列表页面
        setTimeout(() => {
            window.location.href = '/articles';
        }, 1500);
        
    } catch (error) {
        console.error('Create article error:', error);
        
        let errorMessage = '发布文章失败';
        
        if (error.message.includes('401')) {
            errorMessage = '请先登录';
            // 清除无效token
            Auth.logout();
        } else if (error.message.includes('400')) {
            errorMessage = '请检查输入信息是否完整';
        } else {
            errorMessage = `发布失败：${error.message}`;
        }
        
        showMessage(messageElement, errorMessage, 'error');
    } finally {
        // 恢复提交按钮
        submitButton.disabled = false;
        submitButton.textContent = '发布文章';
    }
}

// 实时字数统计（可选功能）
document.addEventListener('DOMContentLoaded', () => {
    const titleInput = document.getElementById('title');
    const contentTextarea = document.getElementById('content');
    
    if (titleInput && contentTextarea) {
        // 创建字数统计元素
        const titleCounter = document.createElement('div');
        titleCounter.className = 'char-counter';
        titleCounter.style.cssText = 'font-size: 0.9rem; color: #666; margin-top: 5px;';
        titleInput.parentNode.appendChild(titleCounter);
        
        const contentCounter = document.createElement('div');
        contentCounter.className = 'char-counter';
        contentCounter.style.cssText = 'font-size: 0.9rem; color: #666; margin-top: 5px;';
        contentTextarea.parentNode.appendChild(contentCounter);
        
        // 更新字数统计
        function updateCounters() {
            const titleLength = titleInput.value.length;
            const contentLength = contentTextarea.value.length;
            
            titleCounter.textContent = `${titleLength}/200 字符`;
            contentCounter.textContent = `${contentLength} 字符`;
            
            // 标题长度警告
            if (titleLength > 200) {
                titleCounter.style.color = '#dc3545';
                titleInput.style.borderColor = '#dc3545';
            } else if (titleLength > 180) {
                titleCounter.style.color = '#ffc107';
                titleInput.style.borderColor = '#ffc107';
            } else {
                titleCounter.style.color = '#666';
                titleInput.style.borderColor = '#e1e5e9';
            }
        }
        
        // 绑定事件
        titleInput.addEventListener('input', updateCounters);
        contentTextarea.addEventListener('input', updateCounters);
        
        // 初始更新
        updateCounters();
    }
});
