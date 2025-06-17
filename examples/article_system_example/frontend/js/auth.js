// 认证相关的 JavaScript

document.addEventListener('DOMContentLoaded', () => {
    // 登录表单处理
    const loginForm = document.getElementById('login-form');
    if (loginForm) {
        loginForm.addEventListener('submit', handleLogin);
    }
    
    // 注册表单处理
    const registerForm = document.getElementById('register-form');
    if (registerForm) {
        registerForm.addEventListener('submit', handleRegister);
    }
    
    // 如果已经登录，重定向到文章页面
    if (Auth.isLoggedIn() && (window.location.pathname.includes('/login') || window.location.pathname.includes('/register'))) {
        window.location.href = '/articles';
    }
});

// 处理登录
async function handleLogin(event) {
    event.preventDefault();
    
    const form = event.target;
    const formData = new FormData(form);
    const loginData = {
        username: formData.get('username'),
        password: formData.get('password')
    };
    
    const messageElement = document.getElementById('login-message');
    const submitButton = form.querySelector('button[type="submit"]');
    
    try {
        // 禁用提交按钮
        submitButton.disabled = true;
        submitButton.textContent = '登录中...';
        
        // 发送登录请求
        const response = await Http.post('/api/auth/login', loginData);
        
        if (response.token) {
            // 登录成功
            Auth.login(response.token);
            
            // 获取用户信息
            try {
                const userInfo = await Http.get('/api/auth/me');
                Storage.set('user_info', userInfo);
            } catch (error) {
                console.error('Failed to fetch user info:', error);
            }
            
            showMessage(messageElement, '登录成功！正在跳转...', 'success');
            
            // 跳转到文章页面
            setTimeout(() => {
                window.location.href = '/articles';
            }, 1000);
        } else {
            showMessage(messageElement, '登录失败：无效的响应', 'error');
        }
    } catch (error) {
        console.error('Login error:', error);
        showMessage(messageElement, `登录失败：${error.message}`, 'error');
    } finally {
        // 恢复提交按钮
        submitButton.disabled = false;  
        submitButton.textContent = '登录';
    }
}

// 处理注册
async function handleRegister(event) {
    event.preventDefault();
    
    const form = event.target;
    const formData = new FormData(form);
    const password = formData.get('password');
    const confirmPassword = formData.get('confirm-password');
    
    const messageElement = document.getElementById('register-message');
    const submitButton = form.querySelector('button[type="submit"]');
    
    // 验证密码
    if (password !== confirmPassword) {
        showMessage(messageElement, '两次输入的密码不一致', 'error');
        return;
    }
    
    if (password.length < 6) {
        showMessage(messageElement, '密码长度不能少于6位', 'error');
        return;
    }
    
    const registerData = {
        username: formData.get('username'),
        email: formData.get('email'),
        password: password
    };
    
    try {
        // 禁用提交按钮
        submitButton.disabled = true;
        submitButton.textContent = '注册中...';
        
        // 发送注册请求
        const response = await Http.post('/api/auth/register', registerData);
        
        showMessage(messageElement, '注册成功！请登录', 'success');
        
        // 清空表单
        form.reset();
        
        // 3秒后跳转到登录页面
        setTimeout(() => {
            window.location.href = '/login';
        }, 2000);
        
    } catch (error) {
        console.error('Register error:', error);
        let errorMessage = '注册失败';
        
        if (error.message.includes('already exists')) {
            errorMessage = '用户名或邮箱已存在';
        } else if (error.message.includes('400')) {
            errorMessage = '请检查输入信息是否完整';
        } else {
            errorMessage = `注册失败：${error.message}`;
        }
        
        showMessage(messageElement, errorMessage, 'error');
    } finally {
        // 恢复提交按钮
        submitButton.disabled = false;
        submitButton.textContent = '注册';
    }
}
