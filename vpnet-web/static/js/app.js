/*
VPNet Web Management Interface - JavaScript

功能包括：
- 主题切换（深色/浅色）
- 侧边栏交互
- 模态框处理
- 页面切换
- 实时数据刷新
- API调用
- 表单处理
- 响应式设计支持
*/

// DOM 加载完成后执行
document.addEventListener('DOMContentLoaded', function() {
    // 初始化主题
    initTheme();
    
    // 初始化侧边栏
    initSidebar();
    
    // 初始化模态框
    initModal();
    
    // 初始化页面切换
    initPageNavigation();
    
    // 初始化实时数据刷新
    initRealTimeUpdates();
    
    // 初始化表单处理
    initForms();
    
    // 初始化事件监听器
    initEventListeners();
    
    // 加载初始数据
    loadInitialData();
});

// 主题管理
function initTheme() {
    const themeToggle = document.getElementById('theme-toggle');
    const currentTheme = localStorage.getItem('theme') || (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light');
    
    // 设置初始主题
    document.documentElement.setAttribute('data-theme', currentTheme);
    
    // 添加主题切换事件
    themeToggle.addEventListener('click', function() {
        const currentTheme = document.documentElement.getAttribute('data-theme');
        const newTheme = currentTheme === 'dark' ? 'light' : 'dark';
        
        // 更新DOM主题
        document.documentElement.setAttribute('data-theme', newTheme);
        
        // 保存主题到localStorage
        localStorage.setItem('theme', newTheme);
    });
}

// 侧边栏管理
function initSidebar() {
    const sidebar = document.getElementById('sidebar');
    const sidebarToggle = document.getElementById('sidebar-toggle');
    const mobileSidebarToggle = document.getElementById('mobile-sidebar-toggle');
    const mainContent = document.getElementById('main-content');
    
    // 侧边栏折叠/展开切换
    sidebarToggle.addEventListener('click', function() {
        sidebar.classList.toggle('collapsed');
    });
    
    // 移动端侧边栏切换
    mobileSidebarToggle.addEventListener('click', function() {
        sidebar.classList.toggle('active');
    });
    
    // 点击主内容区关闭侧边栏（移动端）
    mainContent.addEventListener('click', function(e) {
        if (sidebar.classList.contains('active') && !e.target.closest('.sidebar')) {
            sidebar.classList.remove('active');
        }
    });
    
    // 侧边栏导航链接点击事件
    const navLinks = document.querySelectorAll('.nav-link');
    navLinks.forEach(link => {
        link.addEventListener('click', function(e) {
            e.preventDefault();
            
            // 更新激活状态
            navLinks.forEach(l => l.classList.remove('active'));
            this.classList.add('active');
            
            // 获取目标页面
            const targetPage = this.getAttribute('href').substring(1);
            
            // 切换页面
            switchPage(targetPage);
            
            // 关闭移动端侧边栏
            if (sidebar.classList.contains('active')) {
                sidebar.classList.remove('active');
            }
        });
    });
}

// 页面切换管理
function initPageNavigation() {
    // 监听哈希变化
    window.addEventListener('hashchange', function() {
        const targetPage = window.location.hash.substring(1) || 'dashboard';
        switchPage(targetPage);
    });
}

// 切换页面
function switchPage(pageId) {
    // 更新URL哈希
    window.location.hash = pageId;
    
    // 隐藏所有页面
    const sections = document.querySelectorAll('.section');
    sections.forEach(section => {
        section.classList.remove('active');
    });
    
    // 显示目标页面
    const targetSection = document.getElementById(pageId);
    if (targetSection) {
        targetSection.classList.add('active');
        
        // 触发页面加载事件
        const event = new CustomEvent('pageLoaded', { detail: { pageId } });
        document.dispatchEvent(event);
        
        // 加载页面数据
        loadPageData(pageId);
    }
}

// 模态框管理
function initModal() {
    const modal = document.getElementById('modal');
    const modalOverlay = document.getElementById('modal-overlay');
    const modalClose = document.getElementById('modal-close');
    
    // 点击遮罩层关闭模态框
    modalOverlay.addEventListener('click', function() {
        closeModal();
    });
    
    // 点击关闭按钮关闭模态框
    modalClose.addEventListener('click', function() {
        closeModal();
    });
    
    // 按ESC键关闭模态框
    document.addEventListener('keydown', function(e) {
        if (e.key === 'Escape' && modal.classList.contains('active')) {
            closeModal();
        }
    });
}

// 打开模态框
function openModal(options) {
    const modal = document.getElementById('modal');
    const modalTitle = document.getElementById('modal-title');
    const modalBody = document.getElementById('modal-body');
    const modalFooter = document.getElementById('modal-footer');
    
    // 设置模态框内容
    if (options.title) {
        modalTitle.textContent = options.title;
    }
    
    if (options.body) {
        modalBody.innerHTML = options.body;
    }
    
    if (options.footer) {
        modalFooter.innerHTML = options.footer;
    } else {
        modalFooter.innerHTML = `
            <button class="btn btn-secondary" onclick="closeModal()">取消</button>
            <button class="btn btn-primary" onclick="closeModal()">确定</button>
        `;
    }
    
    // 显示模态框
    modal.classList.add('active');
    
    // 触发模态框打开事件
    const event = new CustomEvent('modalOpened', { detail: options });
    document.dispatchEvent(event);
}

// 关闭模态框
function closeModal() {
    const modal = document.getElementById('modal');
    modal.classList.remove('active');
    
    // 触发模态框关闭事件
    const event = new CustomEvent('modalClosed');
    document.dispatchEvent(event);
}

// 事件监听器初始化
function initEventListeners() {
    // 监听页面加载事件
    document.addEventListener('pageLoaded', function(e) {
        console.log('Page loaded:', e.detail.pageId);
    });
    
    // 监听模态框打开事件
    document.addEventListener('modalOpened', function(e) {
        console.log('Modal opened:', e.detail);
    });
    
    
    // 监听模态框关闭事件
    document.addEventListener('modalClosed', function() {
        console.log('Modal closed');
    });
    
    // 监听表单提交事件
    document.addEventListener('submit', function(e) {
        // 防止默认提交行为
        e.preventDefault();
        
        // 处理表单提交
        handleFormSubmit(e.target);
    });
    
    // 刷新按钮点击事件
    const refreshButtons = document.querySelectorAll('.btn-secondary');
    refreshButtons.forEach(btn => {
        if (btn.querySelector('.fa-sync-alt')) {
            btn.addEventListener('click', function() {
                const currentPage = window.location.hash.substring(1) || 'dashboard';
                loadPageData(currentPage);
            });
        }
    });
    
    // 添加节点按钮点击事件
    const addNodeBtn = document.getElementById('add-node');
    if (addNodeBtn) {
        addNodeBtn.addEventListener('click', function() {
            openModal({
                title: '添加节点',
                body: `
                    <form id="add-node-form">
                        <div class="form-grid">
                            <div class="form-group">
                                <label for="node-name">节点名称</label>
                                <input type="text" id="node-name" name="name" required>
                            </div>
                            <div class="form-group">
                                <label for="node-ip">虚拟IP</label>
                                <input type="text" id="node-ip" name="ip" required>
                            </div>
                            <div class="form-group">
                                <label for="node-subnet">子网掩码</label>
                                <input type="text" id="node-subnet" name="subnet" value="255.255.255.0" required>
                            </div>
                            <div class="form-group">
                                <label for="node-gateway">网关</label>
                                <input type="text" id="node-gateway" name="gateway" value="10.0.0.1" required>
                            </div>
                        </div>
                    </form>
                `,
                footer: `
                    <button class="btn btn-secondary" onclick="closeModal()">取消</button>
                    <button class="btn btn-primary" onclick="handleAddNode()">添加</button>
                `
            });
        });
    }
}

// 表单处理
function initForms() {
    // 处理设置表单
    const settingsForm = document.querySelector('.settings-form');
    if (settingsForm) {
        settingsForm.addEventListener('submit', function(e) {
            e.preventDefault();
            handleSettingsSubmit(this);
        });
    }
}

// 处理表单提交
async function handleFormSubmit(form) {
    console.log('Form submitted:', form.id);
    
    // 获取表单数据
    const formData = new FormData(form);
    const data = Object.fromEntries(formData);
    
    // 根据表单ID处理不同的提交逻辑
    switch(form.id) {
        case 'add-node-form':
            await handleAddNode(data);
            break;
        case 'settings-form':
            await handleSettingsSubmit(data);
            break;
        default:
            console.log('Unknown form:', form.id);
    }
}

// 添加节点处理
async function handleAddNode(data) {
    try {
        // 模拟API调用
        console.log('Adding node:', data);
        
        // 关闭模态框
        closeModal();
        
        // 刷新节点列表
        loadPageData('nodes');
        
        // 显示成功消息
        showNotification('节点添加成功', 'success');
    } catch (error) {
        console.error('Error adding node:', error);
        showNotification('添加节点失败: ' + error.message, 'error');
    }
}

// 设置表单提交处理
async function handleSettingsSubmit(form) {
    try {
        // 获取表单数据
        const formData = new FormData(form);
        const data = Object.fromEntries(formData);
        
        // 模拟API调用
        console.log('Saving settings:', data);
        
        // 显示成功消息
        showNotification('设置保存成功', 'success');
    } catch (error) {
        console.error('Error saving settings:', error);
        showNotification('保存设置失败: ' + error.message, 'error');
    }
}

// 实时数据刷新
function initRealTimeUpdates() {
    // 每30秒刷新一次仪表盘数据
    setInterval(function() {
        const currentPage = window.location.hash.substring(1) || 'dashboard';
        if (currentPage === 'dashboard') {
            loadDashboardData();
        }
    }, 30000);
    
    // 每60秒刷新一次节点数据
    setInterval(function() {
        if (window.location.hash.substring(1) === 'nodes') {
            loadNodesData();
        }
    }, 60000);
}

// 加载初始数据
function loadInitialData() {
    // 加载仪表盘数据
    loadDashboardData();
}

// 加载页面数据
function loadPageData(pageId) {
    switch(pageId) {
        case 'dashboard':
            loadDashboardData();
            break;
        case 'nodes':
            loadNodesData();
            break;
        case 'devices':
            loadDevicesData();
            break;
        case 'routes':
            loadRoutesData();
            break;
        case 'logs':
            loadLogsData();
            break;
        default:
            console.log('No data loader for page:', pageId);
    }
}

// 加载仪表盘数据
async function loadDashboardData() {
    try {
        // 模拟API调用
        const data = await mockApiCall('/api/stats');
        
        // 更新统计数据
        document.getElementById('total-nodes').textContent = data.totalNodes;
        document.getElementById('total-devices').textContent = data.totalDevices;
        document.getElementById('total-routes').textContent = data.totalRoutes;
        document.getElementById('total-traffic').textContent = data.totalTraffic;
        
        console.log('Dashboard data loaded:', data);
    } catch (error) {
        console.error('Error loading dashboard data:', error);
        showNotification('加载仪表盘数据失败', 'error');
    }
}

// 加载节点数据
async function loadNodesData() {
    try {
        // 模拟API调用
        const nodes = await mockApiCall('/api/nodes');
        
        // 更新节点列表
        const tableBody = document.getElementById('nodes-table-body');
        tableBody.innerHTML = '';
        
        nodes.forEach(node => {
            const row = document.createElement('tr');
            row.innerHTML = `
                <td>${node.id}</td>
                <td>${node.name}</td>
                <td><span class="status ${node.status}">${node.status}</span></td>
                <td>${node.virtualIp}</td>
                <td>${node.physicalIp}</td>
                <td>${node.onlineTime}</td>
                <td>
                    <div class="action-buttons">
                        <button class="action-btn edit" title="编辑" onclick="editNode('${node.id}')">
                            <i class="fas fa-edit"></i>
                        </button>
                        <button class="action-btn delete" title="删除" onclick="deleteNode('${node.id}')">
                            <i class="fas fa-trash"></i>
                        </button>
                        <button class="action-btn restart" title="重启" onclick="restartNode('${node.id}')">
                            <i class="fas fa-redo"></i>
                        </button>
                    </div>
                </td>
            `;
            tableBody.appendChild(row);
        });
        
        console.log('Nodes data loaded:', nodes);
    } catch (error) {
        console.error('Error loading nodes data:', error);
        showNotification('加载节点数据失败', 'error');
    }
}

// 加载设备数据
async function loadDevicesData() {
    try {
        // 模拟API调用
        const devices = await mockApiCall('/api/devices');
        
        // 更新设备列表
        const tableBody = document.getElementById('devices-table-body');
        tableBody.innerHTML = '';
        
        devices.forEach(device => {
            const row = document.createElement('tr');
            row.innerHTML = `
                <td>${device.id}</td>
                <td>${device.name}</td>
                <td><span class="status ${device.status}">${device.status}</span></td>
                <td>${device.ip}</td>
                <td>${device.mtu}</td>
                <td>
                    <div class="action-buttons">
                        <button class="action-btn edit" title="编辑" onclick="editDevice('${device.id}')">
                            <i class="fas fa-edit"></i>
                        </button>
                        <button class="action-btn delete" title="删除" onclick="deleteDevice('${device.id}')">
                            <i class="fas fa-trash"></i>
                        </button>
                        <button class="action-btn restart" title="重启" onclick="restartDevice('${device.id}')">
                            <i class="fas fa-redo"></i>
                        </button>
                    </div>
                </td>
            `;
            tableBody.appendChild(row);
        });
        
        console.log('Devices data loaded:', devices);
    } catch (error) {
        console.error('Error loading devices data:', error);
        showNotification('加载设备数据失败', 'error');
    }
}

// 加载路由数据
async function loadRoutesData() {
    try {
        // 模拟API调用
        const routes = await mockApiCall('/api/routes');
        
        // 更新路由列表
        const tableBody = document.getElementById('routes-table-body');
        tableBody.innerHTML = '';
        
        routes.forEach(route => {
            const row = document.createElement('tr');
            row.innerHTML = `
                <td>${route.network}</td>
                <td>${route.mask}</td>
                <td>${route.gateway}</td>
                <td>${route.metric}</td>
                <td>
                    <div class="action-buttons">
                        <button class="action-btn edit" title="编辑" onclick="editRoute('${route.id}')">
                            <i class="fas fa-edit"></i>
                        </button>
                        <button class="action-btn delete" title="删除" onclick="deleteRoute('${route.id}')">
                            <i class="fas fa-trash"></i>
                        </button>
                    </div>
                </td>
            `;
            tableBody.appendChild(row);
        });
        
        console.log('Routes data loaded:', routes);
    } catch (error) {
        console.error('Error loading routes data:', error);
        showNotification('加载路由数据失败', 'error');
    }
}

// 加载日志数据
async function loadLogsData() {
    try {
        // 模拟API调用
        const logs = await mockApiCall('/api/logs');
        
        // 更新日志列表
        const logContainer = document.getElementById('log-container');
        logContainer.innerHTML = '';
        
        logs.forEach(log => {
            const logEntry = document.createElement('div');
            logEntry.className = `log-entry ${log.level}`;
            logEntry.innerHTML = `
                <span class="log-time">${log.time}</span>
                <span class="log-message">${log.message}</span>
            `;
            logContainer.appendChild(logEntry);
        });
        
        // 滚动到底部
        logContainer.scrollTop = logContainer.scrollHeight;
        
        console.log('Logs data loaded:', logs);
    } catch (error) {
        console.error('Error loading logs data:', error);
        showNotification('加载日志数据失败', 'error');
    }
}

// 模拟API调用
async function mockApiCall(endpoint) {
    // 模拟网络延迟
    await new Promise(resolve => setTimeout(resolve, 300));
    
    // 返回模拟数据
    switch(endpoint) {
        case '/api/stats':
            return {
                totalNodes: 5,
                totalDevices: 8,
                totalRoutes: 12,
                totalTraffic: '128.5 MB'
            };
        
        case '/api/nodes':
            return [
                { id: 'node-001', name: 'OpenWrt Router', status: 'online', virtualIp: '10.0.0.1', physicalIp: '192.168.1.1', onlineTime: '2d 14h 30m' },
                { id: 'node-002', name: 'Ubuntu Server', status: 'online', virtualIp: '10.0.0.2', physicalIp: '192.168.0.100', onlineTime: '1d 8h 15m' },
                { id: 'node-003', name: 'Windows PC', status: 'offline', virtualIp: '10.0.0.3', physicalIp: '192.168.2.50', onlineTime: '0h 0m 0s' },
                { id: 'node-004', name: 'macOS Laptop', status: 'online', virtualIp: '10.0.0.4', physicalIp: '192.168.1.100', onlineTime: '5h 20m' },
                { id: 'node-005', name: 'Android Phone', status: 'connecting', virtualIp: '10.0.0.5', physicalIp: '192.168.1.150', onlineTime: '0h 0m 0s' }
            ];
        
        case '/api/devices':
            return [
                { id: 'dev-001', name: 'vpnet0', status: 'up', ip: '10.0.0.1', mtu: 1420 },
                { id: 'dev-002', name: 'vpnet1', status: 'up', ip: '10.0.0.2', mtu: 1420 },
                { id: 'dev-003', name: 'vpnet2', status: 'down', ip: '10.0.0.3', mtu: 1420 },
                { id: 'dev-004', name: 'vpnet3', status: 'up', ip: '10.0.0.4', mtu: 1420 },
                { id: 'dev-005', name: 'vpnet4', status: 'up', ip: '10.0.0.5', mtu: 1420 }
            ];
        
        case '/api/routes':
            return [
                { id: 'route-001', network: '10.0.0.0', mask: '255.255.255.0', gateway: '10.0.0.1', metric: 0 },
                { id: 'route-002', network: '192.168.1.0', mask: '255.255.255.0', gateway: '10.0.0.1', metric: 10 },
                { id: 'route-003', network: '192.168.2.0', mask: '255.255.255.0', gateway: '10.0.0.2', metric: 20 },
                { id: 'route-004', network: '192.168.3.0', mask: '255.255.255.0', gateway: '10.0.0.3', metric: 30 }
            ];
        
        case '/api/logs':
            return [
                { time: '2023-07-01 10:00:00', level: 'info', message: 'Node node-001 connected' },
                { time: '2023-07-01 10:01:30', level: 'info', message: 'Device vpnet0 started' },
                { time: '2023-07-01 10:02:15', level: 'warn', message: 'High CPU usage detected' },
                { time: '2023-07-01 10:03:45', level: 'info', message: 'Route route-001 added' },
                { time: '2023-07-01 10:05:00', level: 'error', message: 'Node node-003 disconnected unexpectedly' }
            ];
        
        default:
            throw new Error(`Unknown endpoint: ${endpoint}`);
    }
}

// 显示通知
function showNotification(message, type = 'info') {
    // 创建通知元素
    const notification = document.createElement('div');
    notification.className = `notification notification-${type}`;
    notification.textContent = message;
    
    // 设置样式
    Object.assign(notification.style, {
        position: 'fixed',
        top: '20px',
        right: '20px',
        padding: '12px 20px',
        borderRadius: '8px',
        color: 'white',
        fontWeight: '500',
        zIndex: '10000',
        boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
        transform: 'translateX(100%)',
        transition: 'transform 0.3s ease-out'
    });
    
    // 根据类型设置背景颜色
    switch(type) {
        case 'success':
            notification.style.backgroundColor = 'var(--success-color)';
            break;
        case 'error':
            notification.style.backgroundColor = 'var(--danger-color)';
            break;
        case 'warn':
            notification.style.backgroundColor = 'var(--warning-color)';
            break;
        default:
            notification.style.backgroundColor = 'var(--info-color)';
    }
    
    // 添加到DOM
    document.body.appendChild(notification);
    
    // 显示通知
    setTimeout(() => {
        notification.style.transform = 'translateX(0)';
    }, 100);
    
    // 3秒后自动隐藏
    setTimeout(() => {
        notification.style.transform = 'translateX(100%)';
        
        // 动画结束后移除元素
        setTimeout(() => {
            document.body.removeChild(notification);
        }, 300);
    }, 3000);
}

// 节点操作函数
function editNode(nodeId) {
    console.log('Edit node:', nodeId);
    openModal({
        title: '编辑节点',
        body: `<p>编辑节点 ${nodeId}</p>`
    });
}

function deleteNode(nodeId) {
    console.log('Delete node:', nodeId);
    openModal({
        title: '删除节点',
        body: `<p>确定要删除节点 ${nodeId} 吗？此操作不可恢复。</p>`,
        footer: `
            <button class="btn btn-secondary" onclick="closeModal()">取消</button>
            <button class="btn btn-danger" onclick="confirmDeleteNode('${nodeId}')">确定删除</button>
        `
    });
}

function confirmDeleteNode(nodeId) {
    console.log('Confirm delete node:', nodeId);
    closeModal();
    showNotification('节点删除成功', 'success');
    loadNodesData();
}

// 设备操作函数
function editDevice(deviceId) {
    console.log('Edit device:', deviceId);
    openModal({
        title: '编辑设备',
        body: `<p>编辑设备 ${deviceId}</p>`
    });
}

function deleteDevice(deviceId) {
    console.log('Delete device:', deviceId);
    openModal({
        title: '删除设备',
        body: `<p>确定要删除设备 ${deviceId} 吗？</p>`,
        footer: `
            <button class="btn btn-secondary" onclick="closeModal()">取消</button>
            <button class="btn btn-danger" onclick="confirmDeleteDevice('${deviceId}')">确定删除</button>
        `
    });
}

function confirmDeleteDevice(deviceId) {
    console.log('Confirm delete device:', deviceId);
    closeModal();
    showNotification('设备删除成功', 'success');
    loadDevicesData();
}

function restartDevice(deviceId) {
    console.log('Restart device:', deviceId);
    showNotification('设备重启中...', 'info');
    setTimeout(() => {
        showNotification('设备重启成功', 'success');
        loadDevicesData();
    }, 2000);
}

// 路由操作函数
function editRoute(routeId) {
    console.log('Edit route:', routeId);
    openModal({
        title: '编辑路由',
        body: `<p>编辑路由 ${routeId}</p>`
    });
}

function deleteRoute(routeId) {
    console.log('Delete route:', routeId);
    openModal({
        title: '删除路由',
        body: `<p>确定要删除路由 ${routeId} 吗？</p>`,
        footer: `
            <button class="btn btn-secondary" onclick="closeModal()">取消</button>
            <button class="btn btn-danger" onclick="confirmDeleteRoute('${routeId}')">确定删除</button>
        `
    });
}

function confirmDeleteRoute(routeId) {
    console.log('Confirm delete route:', routeId);
    closeModal();
    showNotification('路由删除成功', 'success');
    loadRoutesData();
}

// 通用工具函数
function formatBytes(bytes, decimals = 2) {
    if (bytes === 0) return '0 Bytes';
    
    const k = 1024;
    const dm = decimals < 0 ? 0 : decimals;
    const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB', 'PB', 'EB', 'ZB', 'YB'];
    
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    
    return parseFloat((bytes / Math.pow(k, i)).toFixed(dm)) + ' ' + sizes[i];
}

function formatTime(seconds) {
    const d = Math.floor(seconds / (3600 * 24));
    const h = Math.floor((seconds % (3600 * 24)) / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    const s = seconds % 60;
    
    let result = '';
    if (d > 0) result += `${d}d `;
    if (h > 0 || result) result += `${h}h `;
    if (m > 0 || result) result += `${m}m `;
    result += `${s}s`;
    
    return result;
}

function copyToClipboard(text) {
    navigator.clipboard.writeText(text)
        .then(() => {
            showNotification('已复制到剪贴板', 'success');
        })
        .catch(err => {
            console.error('Failed to copy:', err);
            showNotification('复制失败', 'error');
        });
}

// 导出函数，方便外部调用
window.switchPage = switchPage;
window.openModal = openModal;
window.closeModal = closeModal;
window.showNotification = showNotification;
window.copyToClipboard = copyToClipboard;
window.formatBytes = formatBytes;
window.formatTime = formatTime;
