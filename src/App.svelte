<script>
  import { invoke } from '@tauri-apps/api/core';

  // ==================== 状态 ====================
  let reminders = $state([]);
  let categories = $state([]);
  let totalReminders = $state(0);
  let currentPage = $state(1);
  let pageSize = $state(15);
  let totalPages = $state(1);
  let selectedCategory = $state(null);
  let selectedStatus = $state('all');

  // 新任务表单
  let newTitle = $state('');
  let newDescription = $state('');
  let newPriority = $state(1);
  let newCategoryId = $state(null);
  let newDueDate = $state(new Date().toISOString().split('T')[0]);
  let newDueTime = $state('18:00');
  let reminderMode = $state('builtin');
  let selectedReminderFunction = $state(0);
  let customExpression = $state('DueTime-1h');
  let loading = $state(false);

  // 循环任务
  let isRecurring = $state(false);
  let recurrenceType = $state('daily');
  let recurrenceInterval = $state(1);
  let recurrenceDays = $state([]);
  let endType = $state('never');
  let endCount = $state(10);
  let endDate = $state('');
  let recurringTemplates = $state([]);
  let showRecurringModal = $state(false);
  let selectedTemplate = $state(null);
  let templateInstances = $state([]);

  // 弹窗状态
  let showHelp = $state(false);
  let showCategoryModal = $state(false);
  let showImportModal = $state(false);
  let editingCategory = $state(null);
  let newCategoryName = $state('');
  let newCategoryColor = $state('#3b82f6');
  let importJsonText = $state('');
  let importMerge = $state(true);
  let importResult = $state(null);

  // 分页选项
  const pageSizeOptions = [10, 15, 20, 30, 50];

  // 优先级配置
  const priorities = [
    { value: 0, label: '低', color: '#22c55e', bg: '#dcfce7' },
    { value: 1, label: '中', color: '#3b82f6', bg: '#dbeafe' },
    { value: 2, label: '高', color: '#f97316', bg: '#fed7aa' },
    { value: 3, label: '紧急', color: '#ef4444', bg: '#fecaca' }
  ];

  // 预设颜色
  const presetColors = [
    '#3b82f6', '#22c55e', '#f97316', '#ef4444', '#8b5cf6',
    '#ec4899', '#14b8a6', '#f59e0b', '#6b7280', '#0ea5e9'
  ];

  // 时间选项
  const timeOptions = ['08:00', '09:00', '10:00', '11:00', '12:00', '13:00', '14:00', '15:00', '16:00', '17:00', '18:00', '19:00', '20:00', '21:00', '22:00'];

  // 提醒函数
  const reminderFunctions = [
    '完成时间提醒', '提前5分钟', '提前10分钟', '提前15分钟', '提前20分钟', '提前30分钟',
    '提前45分钟', '提前1小时', '提前2小时', '提前3小时', '提前6小时', '提前12小时',
    '提前1天', '提前2天', '提前3天', '提前1周',
    '当天早上7点', '当天早上8点', '当天早上9点', '当天中午12点', '当天傍晚17点', '当天傍晚18点', '当天晚上20点',
    '第二天早上8点', '第二天早上9点'
  ];

  // 循环类型
  const recurrenceTypes = [
    { value: 'daily', label: '每天' },
    { value: 'weekly', label: '每周' },
    { value: 'monthly', label: '每月' },
    { value: 'custom', label: '自定义间隔' }
  ];

  // 星期选项
  const weekdayOptions = [
    { value: 0, label: '周一' },
    { value: 1, label: '周二' },
    { value: 2, label: '周三' },
    { value: 3, label: '周四' },
    { value: 4, label: '周五' },
    { value: 5, label: '周六' },
    { value: 6, label: '周日' }
  ];

  // 月份日期选项（1-31）
  const monthDayOptions = Array.from({ length: 31 }, (_, i) => i + 1);

  // ==================== 加载函数 ====================
  async function loadCategories() {
    try {
      categories = await invoke('get_categories') || [];
    } catch (e) {
      console.error('加载分类失败:', e);
      categories = [];
    }
  }

  async function loadReminders() {
    try {
      const result = await invoke('get_reminders', {
        params: {
          page: currentPage,
          page_size: pageSize,
          category_id: selectedCategory,
          status: selectedStatus
        }
      });
      reminders = result.items || [];
      totalReminders = result.total;
      totalPages = result.total_pages;
    } catch (e) {
      console.error('加载任务失败:', e);
      reminders = [];
      totalReminders = 0;
      totalPages = 1;
    }
  }

  // ==================== 分类操作 ====================
  function openAddCategory() {
    editingCategory = null;
    newCategoryName = '';
    newCategoryColor = '#3b82f6';
    showCategoryModal = true;
  }

  function openEditCategory(cat) {
    editingCategory = cat;
    newCategoryName = cat.name;
    newCategoryColor = cat.color;
    showCategoryModal = true;
  }

  async function saveCategory() {
    if (!newCategoryName.trim()) return;

    try {
      if (editingCategory) {
        await invoke('update_category', {
          id: editingCategory.id,
          name: newCategoryName,
          color: newCategoryColor
        });
      } else {
        const cat = await invoke('add_category', {
          category: { name: newCategoryName, color: newCategoryColor }
        });
        categories = [...categories, cat];
      }
      await loadCategories();
      showCategoryModal = false;
    } catch (e) {
      console.error('保存分类失败:', e);
    }
  }

  async function deleteCategory(id) {
    if (!confirm('确定删除此分类？该分类下的任务将变为无分类。')) return;

    try {
      await invoke('delete_category', { id });
      await loadCategories();
      if (selectedCategory === id) {
        selectedCategory = null;
      }
      await loadReminders();
    } catch (e) {
      console.error('删除分类失败:', e);
    }
  }

  // ==================== 导出导入 ====================
  async function exportData() {
    try {
      const jsonData = await invoke('export_data');
      const blob = new Blob([jsonData], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = `任务提醒数据_${new Date().toISOString().split('T')[0]}.json`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      alert('导出失败: ' + e);
    }
  }

  async function importData() {
    if (!importJsonText.trim()) {
      alert('请输入或粘贴JSON数据');
      return;
    }

    try {
      const result = await invoke('import_data', {
        json_data: importJsonText,
        merge: importMerge
      });
      importResult = result;
      await loadCategories();
      await loadReminders();
      alert(`导入成功！分类: ${result.categories_imported}个，任务: ${result.reminders_imported}个`);
      showImportModal = false;
      importJsonText = '';
      importResult = null;
    } catch (e) {
      alert('导入失败: ' + e);
    }
  }

  function handleFileImport(event) {
    const file = event.target.files[0];
    if (file) {
      const reader = new FileReader();
      reader.onload = (e) => {
        importJsonText = e.target.result;
      };
      reader.readAsText(file);
    }
  }

  // ==================== 任务操作 ====================
  async function addTask() {
    if (!newTitle.trim()) return;
    loading = true;

    const reminderFunction = reminderMode === 'builtin'
      ? reminderFunctions[selectedReminderFunction]
      : customExpression;

    try {
      await invoke('add_reminder', {
        reminder: {
          title: newTitle,
          description: newDescription,
          priority: newPriority,
          category_id: newCategoryId,
          due_time: `${newDueDate}T${newDueTime}`,
          reminder_function: reminderFunction
        }
      });

      // 清空表单
      newTitle = '';
      newDescription = '';
      newPriority = 1;
      newCategoryId = null;

      await loadReminders();
    } catch (e) {
      console.error('添加任务失败:', e);
    }

    loading = false;
  }

  async function completeTask(id) {
    try {
      await invoke('complete_reminder', { id });
      await loadReminders();
    } catch (e) {
      console.error('完成任务失败:', e);
    }
  }

  async function deleteTask(id) {
    if (!confirm('确定删除此任务？')) return;

    try {
      await invoke('delete_reminder', { id });
      await loadReminders();
    } catch (e) {
      console.error('删除任务失败:', e);
    }
  }

  // ==================== 循环任务操作 ====================
  async function loadRecurringTemplates() {
    try {
      recurringTemplates = await invoke('get_recurring_templates') || [];
    } catch (e) {
      console.error('加载循环任务失败:', e);
      recurringTemplates = [];
    }
  }

  async function addRecurringTask() {
    if (!newTitle.trim()) return;
    loading = true;

    try {
      await invoke('add_recurring_template', {
        template: {
          title: newTitle,
          description: newDescription,
          priority: newPriority,
          category_id: newCategoryId,
          base_time: newDueTime,
          recurrence_type: recurrenceType,
          recurrence_interval: recurrenceInterval,
          recurrence_days: recurrenceDays.length > 0 ? recurrenceDays : null,
          end_type: endType,
          end_count: endType === 'count' ? endCount : null,
          end_date: endType === 'date' ? endDate : null
        }
      });

      // 清空表单
      newTitle = '';
      newDescription = '';
      newPriority = 1;
      newCategoryId = null;
      isRecurring = false;
      recurrenceDays = [];

      await loadRecurringTemplates();
      await loadReminders();
    } catch (e) {
      console.error('添加循环任务失败:', e);
      alert('添加失败: ' + e);
    }

    loading = false;
  }

  async function deleteRecurringTemplate(id) {
    if (!confirm('确定删除此循环任务？已生成的任务不会删除。')) return;

    try {
      await invoke('delete_recurring_template', { id });
      await loadRecurringTemplates();
    } catch (e) {
      console.error('删除循环任务失败:', e);
    }
  }

  async function viewTemplateInstances(templateId) {
    try {
      templateInstances = await invoke('get_recurring_instances', {
        template_id: templateId,
        limit: 50
      });
      selectedTemplate = recurringTemplates.find(t => t.id === templateId);
    } catch (e) {
      console.error('加载循环记录失败:', e);
      templateInstances = [];
    }
  }

  function toggleRecurrenceDay(day) {
    if (recurrenceDays.includes(day)) {
      recurrenceDays = recurrenceDays.filter(d => d !== day);
    } else {
      recurrenceDays = [...recurrenceDays, day];
    }
  }

  function getRecurrenceText(template) {
    switch (template.recurrence_type) {
      case 'daily':
        return template.recurrence_interval === 1 ? '每天' : `每${template.recurrence_interval}天`;
      case 'weekly':
        if (template.recurrence_days) {
          const days = JSON.parse(template.recurrence_days);
          const dayNames = days.map(d => weekdayOptions.find(w => w.value === d)?.label || '').join('、');
          return `每周 ${dayNames}`;
        }
        return '每周';
      case 'monthly':
        if (template.recurrence_days) {
          const days = JSON.parse(template.recurrence_days);
          return `每月 ${days.join('、')}号`;
        }
        return '每月';
      case 'custom':
        return `每隔${template.recurrence_interval}天`;
      default:
        return '循环任务';
    }
  }

  // ==================== 分页 ====================
  function goToPage(page) {
    if (page >= 1 && page <= totalPages) {
      currentPage = page;
      loadReminders();
    }
  }

  function changePageSize(size) {
    pageSize = size;
    currentPage = 1;
    loadReminders();
  }

  // 状态筛选
  function filterByStatus(status) {
    selectedStatus = status;
    currentPage = 1;
    loadReminders();
  }

  // 分类筛选
  function filterByCategory(catId) {
    selectedCategory = catId;
    currentPage = 1;
    loadReminders();
  }

  // ==================== 辅助函数 ====================
  function getRemaining(dueTime, completed) {
    if (completed) return '已完成';
    const diff = new Date(dueTime) - new Date();
    if (diff < 0) return '已过期';
    const mins = Math.floor(diff / 60000);
    const hrs = Math.floor(diff / 3600000);
    const days = Math.floor(diff / 86400000);
    if (mins < 60) return `${mins}分钟后`;
    if (hrs < 24) return `${hrs}小时后`;
    return `${days}天后`;
  }

  function formatDate(str) {
    const d = new Date(str);
    return `${d.getMonth()+1}/${d.getDate()} ${String(d.getHours()).padStart(2,'0')}:${String(d.getMinutes()).padStart(2,'0')}`;
  }

  function getPriority(val) {
    return priorities.find(p => p.value === val) || priorities[1];
  }

  function getCategory(id) {
    return categories.find(c => c.id === id);
  }

  // 计算提醒时间
  function calculateReminderTime() {
    const dueDate = new Date(`${newDueDate}T${newDueTime}`);
    let reminderTime = new Date(dueDate);

    if (reminderMode === 'builtin') {
      const func = reminderFunctions[selectedReminderFunction];
      switch (func) {
        case '完成时间提醒': break;
        case '提前5分钟': reminderTime = new Date(dueDate.getTime() - 5 * 60000); break;
        case '提前10分钟': reminderTime = new Date(dueDate.getTime() - 10 * 60000); break;
        case '提前15分钟': reminderTime = new Date(dueDate.getTime() - 15 * 60000); break;
        case '提前20分钟': reminderTime = new Date(dueDate.getTime() - 20 * 60000); break;
        case '提前30分钟': reminderTime = new Date(dueDate.getTime() - 30 * 60000); break;
        case '提前45分钟': reminderTime = new Date(dueDate.getTime() - 45 * 60000); break;
        case '提前1小时': reminderTime = new Date(dueDate.getTime() - 60 * 60000); break;
        case '提前2小时': reminderTime = new Date(dueDate.getTime() - 120 * 60000); break;
        case '提前3小时': reminderTime = new Date(dueDate.getTime() - 180 * 60000); break;
        case '提前6小时': reminderTime = new Date(dueDate.getTime() - 360 * 60000); break;
        case '提前12小时': reminderTime = new Date(dueDate.getTime() - 720 * 60000); break;
        case '提前1天': reminderTime = new Date(dueDate.getTime() - 1440 * 60000); break;
        case '提前2天': reminderTime = new Date(dueDate.getTime() - 2880 * 60000); break;
        case '提前3天': reminderTime = new Date(dueDate.getTime() - 4320 * 60000); break;
        case '提前1周': reminderTime = new Date(dueDate.getTime() - 10080 * 60000); break;
        case '当天早上7点': reminderTime = new Date(`${newDueDate}T07:00`); break;
        case '当天早上8点': reminderTime = new Date(`${newDueDate}T08:00`); break;
        case '当天早上9点': reminderTime = new Date(`${newDueDate}T09:00`); break;
        case '当天中午12点': reminderTime = new Date(`${newDueDate}T12:00`); break;
        case '当天傍晚17点': reminderTime = new Date(`${newDueDate}T17:00`); break;
        case '当天傍晚18点': reminderTime = new Date(`${newDueDate}T18:00`); break;
        case '当天晚上20点': reminderTime = new Date(`${newDueDate}T20:00`); break;
        case '第二天早上8点':
          const tomorrow = new Date(dueDate);
          tomorrow.setDate(tomorrow.getDate() + 1);
          reminderTime = new Date(`${tomorrow.toISOString().split('T')[0]}T08:00`);
          break;
        case '第二天早上9点':
          const tomorrow2 = new Date(dueDate);
          tomorrow2.setDate(tomorrow2.getDate() + 1);
          reminderTime = new Date(`${tomorrow2.toISOString().split('T')[0]}T09:00`);
          break;
      }
    } else {
      const expr = customExpression.trim().toLowerCase();
      reminderTime = parseCustomExpression(dueDate, expr);
    }

    return reminderTime;
  }

  function parseCustomExpression(dueDate, expr) {
    let result = new Date(dueDate);

    if (expr.startsWith('duetime')) {
      const offset = expr.substring(7).trim();
      result = applyOffset(dueDate, offset);
    } else if (expr.startsWith('date')) {
      const offset = expr.substring(4).trim();
      const baseDate = new Date(`${newDueDate}T00:00`);
      result = applyOffset(baseDate, offset);
    } else if (expr.startsWith('tomorrow')) {
      const offset = expr.substring(8).trim();
      const tomorrow = new Date(dueDate);
      tomorrow.setDate(tomorrow.getDate() + 1);
      const baseDate = new Date(`${tomorrow.toISOString().split('T')[0]}T00:00`);
      result = applyOffset(baseDate, offset);
    } else if (expr.startsWith('nextworkday')) {
      const offset = expr.substring(11).trim();
      let nextDay = new Date(dueDate);
      nextDay.setDate(nextDay.getDate() + 1);
      while (nextDay.getDay() === 0 || nextDay.getDay() === 6) {
        nextDay.setDate(nextDay.getDate() + 1);
      }
      const baseDate = new Date(`${nextDay.toISOString().split('T')[0]}T00:00`);
      result = applyOffset(baseDate, offset);
    }

    return result;
  }

  function applyOffset(baseDate, offset) {
    let result = new Date(baseDate);
    if (!offset) return result;

    const sign = offset.startsWith('+') ? 1 : offset.startsWith('-') ? -1 : 0;
    const numStr = offset.replace(/[+-]/, '').replace(/[mhd]$/, '');
    const num = parseInt(numStr) || 0;
    const unit = offset.slice(-1);

    if (unit === 'm') {
      result = new Date(result.getTime() + sign * num * 60000);
    } else if (unit === 'h') {
      result = new Date(result.getTime() + sign * num * 3600000);
    } else if (unit === 'd') {
      result = new Date(result.getTime() + sign * num * 86400000);
    }

    return result;
  }

  function formatReminderTime(date) {
    const month = date.getMonth() + 1;
    const day = date.getDate();
    const hour = String(date.getHours()).padStart(2, '0');
    const minute = String(date.getMinutes()).padStart(2, '0');
    const weekdays = ['周日', '周一', '周二', '周三', '周四', '周五', '周六'];
    const weekday = weekdays[date.getDay()];
    return `${month}月${day}日 ${hour}:${minute} ${weekday}`;
  }

  let reminderTimeDisplay = $derived(() => {
    const reminderTime = calculateReminderTime();
    return formatReminderTime(reminderTime);
  });

  // 初始化
  loadCategories();
  loadReminders();
  loadRecurringTemplates();
</script>

<main style="min-height: 100vh; padding: 20px 30px; background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); font-family: 'Segoe UI', system-ui, sans-serif;">
  <div style="max-width: 1100px; margin: 0 auto;">

    <!-- 标题栏 -->
    <header style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 24px;">
      <div style="display: flex; align-items: center; gap: 14px;">
        <div style="width: 48px; height: 48px; background: white; border-radius: 14px; display: flex; align-items: center; justify-content: center; box-shadow: 0 4px 12px rgba(0,0,0,0.15);">
          <span style="font-size: 24px;">⏰</span>
        </div>
        <div>
          <h1 style="font-size: 28px; font-weight: 700; color: white; letter-spacing: -0.5px; margin: 0;">任务提醒助手</h1>
          <div style="color: rgba(255,255,255,0.7); font-size: 11px; margin-top: 2px;">浙江巨鼎包装有限公司 · 开发：应圣卫</div>
        </div>
      </div>
      <div style="display: flex; align-items: center; gap: 12px;">
        <button onclick={() => showRecurringModal = true} style="padding: 8px 16px; background: rgba(255,255,255,0.2); color: white; border-radius: 20px; border: none; cursor: pointer; font-size: 13px; font-weight: 600; transition: all 0.2s;">🔄 循环任务</button>
        <button onclick={exportData} style="padding: 8px 16px; background: rgba(255,255,255,0.2); color: white; border-radius: 20px; border: none; cursor: pointer; font-size: 13px; font-weight: 600; transition: all 0.2s;">📤 导出</button>
        <button onclick={() => showImportModal = true} style="padding: 8px 16px; background: rgba(255,255,255,0.2); color: white; border-radius: 20px; border: none; cursor: pointer; font-size: 13px; font-weight: 600; transition: all 0.2s;">📥 导入</button>
        <button onclick={() => showHelp = true} style="padding: 8px 16px; background: rgba(255,255,255,0.2); color: white; border-radius: 20px; border: none; cursor: pointer; font-size: 13px; font-weight: 600; transition: all 0.2s;">📖 帮助</button>
      </div>
    </header>

    <div style="display: grid; grid-template-columns: 400px 1fr; gap: 20px;">

      <!-- 左侧：添加任务 -->
      <div style="background: white; border-radius: 20px; padding: 28px; box-shadow: 0 8px 32px rgba(0,0,0,0.12);">
        <h2 style="font-size: 20px; font-weight: 700; color: #1e293b; margin: 0 0 24px 0; display: flex; align-items: center; gap: 10px;">
          <span style="width: 32px; height: 32px; background: linear-gradient(135deg, #667eea, #764ba2); border-radius: 8px; display: flex; align-items: center; justify-content: center; color: white; font-size: 18px;">+</span>
          添加新任务
        </h2>

        <!-- 标题 -->
        <div style="margin-bottom: 18px;">
          <label style="display: block; font-size: 13px; font-weight: 600; color: #475569; margin-bottom: 8px;">任务标题 *</label>
          <input type="text" bind:value={newTitle} placeholder="例如：完成项目报告" style="width: 100%; padding: 14px 16px; border: 2px solid #e2e8f0; border-radius: 12px; font-size: 15px; outline: none; transition: border-color 0.2s; box-sizing: border-box; background: #f8fafc;" />
        </div>

        <!-- 描述 -->
        <div style="margin-bottom: 18px;">
          <label style="display: block; font-size: 13px; font-weight: 600; color: #475569; margin-bottom: 8px;">任务描述</label>
          <textarea bind:value={newDescription} placeholder="详细描述任务内容..." rows="2" style="width: 100%; padding: 14px 16px; border: 2px solid #e2e8f0; border-radius: 12px; font-size: 15px; outline: none; resize: none; box-sizing: border-box; background: #f8fafc;"></textarea>
        </div>

        <!-- 优先级 -->
        <div style="margin-bottom: 18px;">
          <label style="display: block; font-size: 13px; font-weight: 600; color: #475569; margin-bottom: 10px;">优先级</label>
          <div style="display: flex; gap: 10px;">
            {#each priorities as p}
              <button onclick={() => newPriority = p.value} style="flex: 1; padding: 10px 0; border-radius: 10px; font-size: 14px; font-weight: 600; cursor: pointer; border: 2px solid transparent; transition: all 0.2s; background: {newPriority === p.value ? p.color : p.bg}; color: {newPriority === p.value ? 'white' : p.color}; box-shadow: {newPriority === p.value ? '0 4px 12px ' + p.color + '40' : 'none'};">{p.label}</button>
            {/each}
          </div>
        </div>

        <!-- 分类 -->
        <div style="margin-bottom: 18px;">
          <label style="display: block; font-size: 13px; font-weight: 600; color: #475569; margin-bottom: 8px;">分类</label>
          <select bind:value={newCategoryId} style="width: 100%; padding: 12px 14px; border: 2px solid #e2e8f0; border-radius: 12px; font-size: 14px; outline: none; background: #f8fafc; cursor: pointer;">
            <option value={null}>无分类</option>
            {#each categories as cat}
              <option value={cat.id}>{cat.name}</option>
            {/each}
          </select>
        </div>

        <!-- 完成时间 -->
        <div style="margin-bottom: 18px;">
          <label style="display: block; font-size: 13px; font-weight: 600; color: #475569; margin-bottom: 8px;">完成时间</label>
          <div style="display: flex; gap: 10px;">
            <input type="date" bind:value={newDueDate} style="flex: 1; padding: 12px 14px; border: 2px solid #e2e8f0; border-radius: 12px; font-size: 14px; outline: none; background: #f8fafc;" />
            <select bind:value={newDueTime} style="width: 100px; padding: 12px 14px; border: 2px solid #e2e8f0; border-radius: 12px; font-size: 14px; outline: none; background: #f8fafc; cursor: pointer;">
              {#each timeOptions as t}
                <option value={t}>{t}</option>
              {/each}
            </select>
          </div>
        </div>

        <!-- 提醒方式 -->
        <div style="margin-bottom: 18px;">
          <label style="display: block; font-size: 13px; font-weight: 600; color: #475569; margin-bottom: 10px;">提醒方式</label>
          <div style="display: flex; gap: 10px; margin-bottom: 12px;">
            <button onclick={() => reminderMode = 'builtin'} style="flex: 1; padding: 10px 0; border-radius: 10px; font-size: 14px; font-weight: 600; cursor: pointer; border: none; background: {reminderMode === 'builtin' ? 'linear-gradient(135deg, #667eea, #764ba2)' : '#f1f5f9'}; color: {reminderMode === 'builtin' ? 'white' : '#64748b'};">内置函数</button>
            <button onclick={() => reminderMode = 'custom'} style="flex: 1; padding: 10px 0; border-radius: 10px; font-size: 14px; font-weight: 600; cursor: pointer; border: none; background: {reminderMode === 'custom' ? 'linear-gradient(135deg, #667eea, #764ba2)' : '#f1f5f9'}; color: {reminderMode === 'custom' ? 'white' : '#64748b'};">自定义公式</button>
          </div>

          {#if reminderMode === 'builtin'}
            <select bind:value={selectedReminderFunction} style="width: 100%; padding: 12px 14px; border: 2px solid #e2e8f0; border-radius: 12px; font-size: 14px; outline: none; background: #f8fafc; cursor: pointer;">
              {#each reminderFunctions as f, i}
                <option value={i}>{f}</option>
              {/each}
            </select>
          {:else}
            <input type="text" bind:value={customExpression} placeholder="例如：DueTime-1h" style="width: 100%; padding: 12px 14px; border: 2px solid #e2e8f0; border-radius: 12px; font-size: 14px; outline: none; background: #f8fafc;" />
            <div style="margin-top: 10px; padding: 12px; background: #f1f5f9; border-radius: 10px; font-size: 12px; color: #64748b; line-height: 1.6;">
              <div style="display: flex; justify-content: space-between; align-items: center;">
                <span style="font-weight: 600; color: #475569;">可用表达式（点击帮助查看详情）</span>
              </div>
              <div style="margin-top: 6px;">• DueTime-1h / Date+9h / Tomorrow+9h</div>
            </div>
          {/if}
        </div>

        <!-- 提醒预览 -->
        {#if !isRecurring}
        <div style="background: linear-gradient(135deg, #667eea, #764ba2); border-radius: 14px; padding: 18px; margin-bottom: 20px; box-shadow: 0 4px 16px rgba(102,126,234,0.3);">
          <div style="color: rgba(255,255,255,0.85); font-size: 13px; margin-bottom: 6px;">
            提醒时间预览
            <span style="margin-left: 8px; padding: 2px 8px; background: rgba(255,255,255,0.2); border-radius: 4px; font-size: 11px;">
              {reminderMode === 'builtin' ? reminderFunctions[selectedReminderFunction] : customExpression}
            </span>
          </div>
          <div style="color: white; font-size: 22px; font-weight: 700;">{reminderTimeDisplay()}</div>
        </div>
        {/if}

        <!-- 循环任务设置 -->
        <div style="margin-bottom: 18px; padding: 16px; background: #f0f9ff; border-radius: 12px; border: 2px solid #bae6fd;">
          <label style="display: flex; align-items: center; gap: 10px; cursor: pointer; margin-bottom: 10px;">
            <input type="checkbox" bind:checked={isRecurring} style="width: 18px; height: 18px;" />
            <span style="font-size: 14px; font-weight: 600; color: #0369a1;">🔄 设为循环任务</span>
          </label>

          {#if isRecurring}
            <div style="margin-top: 12px; padding-top: 12px; border-top: 1px solid #bae6fd;">
              <!-- 循环类型 -->
              <div style="margin-bottom: 12px;">
                <label style="display: block; font-size: 12px; font-weight: 600; color: #475569; margin-bottom: 6px;">循环周期</label>
                <select bind:value={recurrenceType} style="width: 100%; padding: 10px 12px; border: 2px solid #e2e8f0; border-radius: 8px; font-size: 13px; background: white;">
                  {#each recurrenceTypes as rt}
                    <option value={rt.value}>{rt.label}</option>
                  {/each}
                </select>
              </div>

              <!-- 间隔 -->
              {#if recurrenceType === 'daily' || recurrenceType === 'custom'}
                <div style="margin-bottom: 12px;">
                  <label style="display: block; font-size: 12px; font-weight: 600; color: #475569; margin-bottom: 6px;">
                    间隔（每几天）
                  </label>
                  <input type="number" bind:value={recurrenceInterval} min="1" max="365" style="width: 100%; padding: 10px 12px; border: 2px solid #e2e8f0; border-radius: 8px; font-size: 13px; background: white;" />
                </div>
              {/if}

              <!-- 每周几 -->
              {#if recurrenceType === 'weekly'}
                <div style="margin-bottom: 12px;">
                  <label style="display: block; font-size: 12px; font-weight: 600; color: #475569; margin-bottom: 6px;">选择星期</label>
                  <div style="display: flex; gap: 6px; flex-wrap: wrap;">
                    {#each weekdayOptions as wd}
                      <button onclick={() => toggleRecurrenceDay(wd.value)} style="padding: 6px 12px; border-radius: 6px; font-size: 12px; font-weight: 600; cursor: pointer; border: none; background: {recurrenceDays.includes(wd.value) ? '#0ea5e9' : '#e2e8f0'}; color: {recurrenceDays.includes(wd.value) ? 'white' : '#64748b'};">{wd.label}</button>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- 每月几号 -->
              {#if recurrenceType === 'monthly'}
                <div style="margin-bottom: 12px;">
                  <label style="display: block; font-size: 12px; font-weight: 600; color: #475569; margin-bottom: 6px;">选择日期（可多选）</label>
                  <div style="display: flex; gap: 4px; flex-wrap: wrap; max-height: 80px; overflow-y: auto;">
                    {#each monthDayOptions as day}
                      <button onclick={() => toggleRecurrenceDay(day)} style="width: 32px; height: 32px; border-radius: 6px; font-size: 11px; font-weight: 600; cursor: pointer; border: none; background: {recurrenceDays.includes(day) ? '#0ea5e9' : '#e2e8f0'}; color: {recurrenceDays.includes(day) ? 'white' : '#64748b'};">{day}</button>
                    {/each}
                  </div>
                </div>
              {/if}

              <!-- 结束条件 -->
              <div style="margin-bottom: 12px;">
                <label style="display: block; font-size: 12px; font-weight: 600; color: #475569; margin-bottom: 6px;">结束条件</label>
                <div style="display: flex; gap: 8px;">
                  <button onclick={() => endType = 'never'} style="flex: 1; padding: 8px; border-radius: 6px; font-size: 12px; font-weight: 600; cursor: pointer; border: none; background: {endType === 'never' ? '#0ea5e9' : '#e2e8f0'}; color: {endType === 'never' ? 'white' : '#64748b'};">永不</button>
                  <button onclick={() => endType = 'count'} style="flex: 1; padding: 8px; border-radius: 6px; font-size: 12px; font-weight: 600; cursor: pointer; border: none; background: {endType === 'count' ? '#0ea5e9' : '#e2e8f0'}; color: {endType === 'count' ? 'white' : '#64748b'};">次数</button>
                  <button onclick={() => endType = 'date'} style="flex: 1; padding: 8px; border-radius: 6px; font-size: 12px; font-weight: 600; cursor: pointer; border: none; background: {endType === 'date' ? '#0ea5e9' : '#e2e8f0'}; color: {endType === 'date' ? 'white' : '#64748b'};">日期</button>
                </div>
              </div>

              {#if endType === 'count'}
                <div style="margin-bottom: 12px;">
                  <label style="display: block; font-size: 12px; font-weight: 600; color: #475569; margin-bottom: 6px;">重复次数</label>
                  <input type="number" bind:value={endCount} min="1" max="999" style="width: 100%; padding: 10px 12px; border: 2px solid #e2e8f0; border-radius: 8px; font-size: 13px; background: white;" />
                </div>
              {/if}

              {#if endType === 'date'}
                <div style="margin-bottom: 12px;">
                  <label style="display: block; font-size: 12px; font-weight: 600; color: #475569; margin-bottom: 6px;">结束日期</label>
                  <input type="date" bind:value={endDate} style="width: 100%; padding: 10px 12px; border: 2px solid #e2e8f0; border-radius: 8px; font-size: 13px; background: white;" />
                </div>
              {/if}
            </div>
          {/if}
        </div>

        <!-- 添加按钮 -->
        <button onclick={isRecurring ? addRecurringTask : addTask} disabled={loading || !newTitle.trim()} style="width: 100%; padding: 16px; background: linear-gradient(135deg, #667eea, #764ba2); color: white; font-size: 16px; font-weight: 700; border-radius: 14px; border: none; cursor: pointer; opacity: {loading || !newTitle.trim() ? '0.6' : '1'}; transition: all 0.2s; box-shadow: 0 4px 16px rgba(102,126,234,0.4);">
          {loading ? '添加中...' : isRecurring ? '+ 添加循环任务' : '+ 添加任务'}
        </button>
      </div>

      <!-- 右侧：任务列表 -->
      <div style="background: white; border-radius: 20px; padding: 28px; box-shadow: 0 8px 32px rgba(0,0,0,0.12); min-height: 600px;">

        <!-- 分类管理 -->
        <div style="display: flex; align-items: center; gap: 10px; margin-bottom: 16px;">
          <button onclick={openAddCategory} style="padding: 8px 14px; background: linear-gradient(135deg, #667eea, #764ba2); color: white; border-radius: 10px; border: none; cursor: pointer; font-size: 13px; font-weight: 600;">+ 新建</button>

          <!-- 分类列表（可编辑） -->
          <div style="display: flex; gap: 6px; flex-wrap: wrap;">
            <button onclick={() => filterByCategory(null)} style="padding: 6px 12px; border-radius: 8px; font-size: 12px; font-weight: 600; cursor: pointer; border: none; background: {selectedCategory === null ? 'linear-gradient(135deg, #667eea, #764ba2)' : '#f1f5f9'}; color: {selectedCategory === null ? 'white' : '#64748b'};">全部</button>
            {#each categories as cat}
              <div style="display: flex; align-items: center; gap: 2px;">
                <button onclick={() => filterByCategory(cat.id)} style="padding: 6px 12px; border-radius: 8px 0 0 8px; font-size: 12px; font-weight: 600; cursor: pointer; border: none; background: {selectedCategory === cat.id ? cat.color : '#f1f5f9'}; color: {selectedCategory === cat.id ? 'white' : '#64748b'};">{cat.name}</button>
                <button onclick={() => openEditCategory(cat)} style="padding: 6px 8px; border-radius: 0 8px 8px 0; font-size: 12px; cursor: pointer; border: none; background: {selectedCategory === cat.id ? cat.color : '#e2e8f0'}; color: {selectedCategory === cat.id ? 'white' : '#64748b'};" title="编辑分类">✏️</button>
              </div>
            {/each}
          </div>
        </div>

        <!-- 状态筛选 -->
        <div style="display: flex; gap: 10px; margin-bottom: 20px;">
          <button onclick={() => filterByStatus('all')} style="flex: 1; padding: 10px; border-radius: 12px; font-size: 14px; font-weight: 600; cursor: pointer; border: none; background: {selectedStatus === 'all' ? 'linear-gradient(135deg, #667eea, #764ba2)' : '#f1f5f9'}; color: {selectedStatus === 'all' ? 'white' : '#64748b'};">全部 ({totalReminders})</button>
          <button onclick={() => filterByStatus('pending')} style="flex: 1; padding: 10px; border-radius: 12px; font-size: 14px; font-weight: 600; cursor: pointer; border: none; background: {selectedStatus === 'pending' ? '#f97316' : '#f1f5f9'}; color: {selectedStatus === 'pending' ? 'white' : '#64748b'};">待办</button>
          <button onclick={() => filterByStatus('completed')} style="flex: 1; padding: 10px; border-radius: 12px; font-size: 14px; font-weight: 600; cursor: pointer; border: none; background: {selectedStatus === 'completed' ? '#22c55e' : '#f1f5f9'}; color: {selectedStatus === 'completed' ? 'white' : '#64748b'};">已完成</button>
        </div>

        <h2 style="font-size: 18px; font-weight: 700; color: #1e293b; margin: 0 0 16px 0;">任务列表</h2>

        <!-- 任务列表 -->
        <div style="display: flex; flex-direction: column; gap: 12px; max-height: 400px; overflow-y: auto; padding-right: 8px;">
          {#each reminders as reminder (reminder.id)}
            <div style="background: #f8fafc; border: 2px solid #e2e8f0; border-radius: 14px; padding: 18px; transition: all 0.2s; {reminder.is_completed ? 'opacity: 0.6;' : ''}">
              <div style="display: flex; align-items: flex-start; gap: 14px;">
                <!-- 优先级 -->
                <div style="width: 14px; height: 14px; border-radius: 50%; background: {getPriority(reminder.priority).color}; margin-top: 4px; box-shadow: 0 2px 8px {getPriority(reminder.priority).color}40;"></div>
                <!-- 内容 -->
                <div style="flex: 1; min-width: 0;">
                  <h3 style="font-size: 16px; font-weight: 700; color: #1e293b; margin: 0 0 4px 0; {reminder.is_completed ? 'text-decoration: line-through;' : ''}">{reminder.template_id ? '🔄 ' : ''}{reminder.title}</h3>
                  {#if reminder.description}
                    <p style="font-size: 13px; color: #64748b; margin: 0 0 8px 0;">{reminder.description}</p>
                  {/if}
                  <div style="display: flex; align-items: center; gap: 10px; font-size: 12px;">
                    {#if reminder.category_name}
                      <span style="background: {reminder.category_color || '#3b82f6'}20; color: {reminder.category_color || '#3b82f6'}; padding: 4px 10px; border-radius: 6px; font-weight: 600;">{reminder.category_name}</span>
                    {/if}
                    <span style="background: {getPriority(reminder.priority).bg}; color: {getPriority(reminder.priority).color}; padding: 4px 10px; border-radius: 6px; font-weight: 600;">{getPriority(reminder.priority).label}</span>
                    <span style="color: #94a3b8;">{formatDate(reminder.due_time)}</span>
                    <span style="color: {reminder.is_completed ? '#22c55e' : new Date(reminder.due_time) < new Date() ? '#ef4444' : '#f97316'}; font-weight: 600;">{getRemaining(reminder.due_time, reminder.is_completed)}</span>
                  </div>
                </div>
                <!-- 操作 -->
                <div style="display: flex; gap: 8px;">
                  {#if !reminder.is_completed}
                    <button onclick={() => completeTask(reminder.id)} style="width: 36px; height: 36px; border-radius: 10px; background: #22c55e; color: white; border: none; cursor: pointer; font-size: 18px; box-shadow: 0 2px 8px #22c55e40;">✓</button>
                  {/if}
                  <button onclick={() => deleteTask(reminder.id)} style="width: 36px; height: 36px; border-radius: 10px; background: #ef4444; color: white; border: none; cursor: pointer; font-size: 18px; box-shadow: 0 2px 8px #ef444440;">×</button>
                </div>
              </div>
            </div>
          {:else}
            <div style="text-align: center; padding: 60px 20px; color: #94a3b8;">
              <span style="font-size: 64px;">📝</span>
              <p style="margin-top: 16px; font-size: 16px;">暂无任务，添加一个试试吧</p>
            </div>
          {/each}
        </div>

        <!-- 分页 -->
        {#if totalPages > 1}
          <div style="display: flex; align-items: center; justify-content: space-between; margin-top: 20px; padding-top: 16px; border-top: 2px solid #e2e8f0;">
            <!-- 左：每页条数 -->
            <div style="display: flex; align-items: center; gap: 8px;">
              <span style="font-size: 13px; color: #64748b;">每页：</span>
              {#each pageSizeOptions as size}
                <button onclick={() => changePageSize(size)} style="padding: 6px 10px; border-radius: 8px; font-size: 12px; font-weight: 600; cursor: pointer; border: none; background: {pageSize === size ? 'linear-gradient(135deg, #667eea, #764ba2)' : '#f1f5f9'}; color: {pageSize === size ? 'white' : '#64748b'};">{size}</button>
              {/each}
            </div>
            <!-- 右：页码 -->
            <div style="display: flex; align-items: center; gap: 8px;">
              <button onclick={() => goToPage(currentPage - 1)} disabled={currentPage === 1} style="padding: 6px 12px; border-radius: 8px; font-size: 12px; cursor: pointer; border: none; background: #f1f5f9; color: #64748b; opacity: {currentPage === 1 ? '0.5' : '1'};">上一页</button>
              {#if currentPage > 2}
                <button onclick={() => goToPage(1)} style="padding: 6px 10px; border-radius: 8px; font-size: 12px; font-weight: 600; cursor: pointer; border: none; background: #f1f5f9; color: #64748b;">1</button>
                <span style="color: #94a3b8;">...</span>
              {/if}
              {#if currentPage > 1}
                <button onclick={() => goToPage(currentPage - 1)} style="padding: 6px 10px; border-radius: 8px; font-size: 12px; font-weight: 600; cursor: pointer; border: none; background: #f1f5f9; color: #64748b;">{currentPage - 1}</button>
              {/if}
              <span style="padding: 6px 10px; border-radius: 8px; font-size: 12px; font-weight: 700; background: linear-gradient(135deg, #667eea, #764ba2); color: white;">{currentPage}</span>
              {#if currentPage < totalPages}
                <button onclick={() => goToPage(currentPage + 1)} style="padding: 6px 10px; border-radius: 8px; font-size: 12px; font-weight: 600; cursor: pointer; border: none; background: #f1f5f9; color: #64748b;">{currentPage + 1}</button>
              {/if}
              {#if currentPage < totalPages - 1}
                <span style="color: #94a3b8;">...</span>
                <button onclick={() => goToPage(totalPages)} style="padding: 6px 10px; border-radius: 8px; font-size: 12px; font-weight: 600; cursor: pointer; border: none; background: #f1f5f9; color: #64748b;">{totalPages}</button>
              {/if}
              <button onclick={() => goToPage(currentPage + 1)} disabled={currentPage === totalPages} style="padding: 6px 12px; border-radius: 8px; font-size: 12px; cursor: pointer; border: none; background: #f1f5f9; color: #64748b; opacity: {currentPage === totalPages ? '0.5' : '1'};">下一页</button>
            </div>
          </div>
        {/if}
      </div>
    </div>

    <!-- 分类管理弹窗 -->
  {#if showCategoryModal}
    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;" onclick={() => showCategoryModal = false}>
      <div style="background: white; border-radius: 20px; padding: 28px; width: 360px; box-shadow: 0 20px 60px rgba(0,0,0,0.3);" onclick={(e) => e.stopPropagation()}>
        <h2 style="font-size: 20px; font-weight: 700; color: #1e293b; margin: 0 0 20px 0;">{editingCategory ? '编辑分类' : '新建分类'}</h2>

        <div style="margin-bottom: 16px;">
          <label style="display: block; font-size: 13px; font-weight: 600; color: #475569; margin-bottom: 8px;">分类名称</label>
          <input type="text" bind:value={newCategoryName} placeholder="输入分类名称" style="width: 100%; padding: 12px 14px; border: 2px solid #e2e8f0; border-radius: 10px; font-size: 14px; outline: none; box-sizing: border-box;" />
        </div>

        <div style="margin-bottom: 20px;">
          <label style="display: block; font-size: 13px; font-weight: 600; color: #475569; margin-bottom: 8px;">分类颜色</label>
          <div style="display: flex; gap: 8px; flex-wrap: wrap;">
            {#each presetColors as color}
              <button onclick={() => newCategoryColor = color} style="width: 32px; height: 32px; border-radius: 8px; border: 3px solid {newCategoryColor === color ? '#1e293b' : 'transparent'}; background: {color}; cursor: pointer;"></button>
            {/each}
          </div>
        </div>

        <div style="display: flex; gap: 10px;">
          <button onclick={() => showCategoryModal = false} style="flex: 1; padding: 12px; border-radius: 12px; font-size: 14px; font-weight: 600; cursor: pointer; border: none; background: #f1f5f9; color: #64748b;">取消</button>
          <button onclick={saveCategory} disabled={!newCategoryName.trim()} style="flex: 1; padding: 12px; border-radius: 12px; font-size: 14px; font-weight: 600; cursor: pointer; border: none; background: linear-gradient(135deg, #667eea, #764ba2); color: white; opacity: {newCategoryName.trim() ? '1' : '0.5'};">保存</button>
        </div>

        {#if editingCategory}
          <button onclick={() => { deleteCategory(editingCategory.id); showCategoryModal = false; }} style="width: 100%; margin-top: 12px; padding: 10px; border-radius: 10px; font-size: 13px; font-weight: 600; cursor: pointer; border: none; background: #fef2f2; color: #ef4444;">删除此分类</button>
        {/if}
      </div>
    </div>
  {/if}

  <!-- 导入弹窗 -->
  {#if showImportModal}
    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;" onclick={() => showImportModal = false}>
      <div style="background: white; border-radius: 20px; padding: 28px; width: 500px; max-height: 80vh; overflow-y: auto; box-shadow: 0 20px 60px rgba(0,0,0,0.3);" onclick={(e) => e.stopPropagation()}>
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
          <h2 style="font-size: 20px; font-weight: 700; color: #1e293b; margin: 0;">📥 导入数据</h2>
          <button onclick={() => showImportModal = false} style="width: 36px; height: 36px; border-radius: 50%; background: #f1f5f9; border: none; cursor: pointer; font-size: 20px; color: #64748b;">×</button>
        </div>

        <div style="margin-bottom: 16px;">
          <label style="display: block; font-size: 13px; font-weight: 600; color: #475569; margin-bottom: 8px;">选择文件</label>
          <input type="file" accept=".json" onchange={handleFileImport} style="width: 100%; padding: 10px; border: 2px dashed #e2e8f0; border-radius: 10px; font-size: 14px; background: #f8fafc; cursor: pointer;" />
        </div>

        <div style="margin-bottom: 16px;">
          <label style="display: block; font-size: 13px; font-weight: 600; color: #475569; margin-bottom: 8px;">或粘贴JSON数据</label>
          <textarea bind:value={importJsonText} placeholder="粘贴导出的JSON数据..." rows="6" style="width: 100%; padding: 12px; border: 2px solid #e2e8f0; border-radius: 10px; font-size: 13px; outline: none; resize: none; box-sizing: border-box; font-family: monospace;"></textarea>
        </div>

        <div style="margin-bottom: 20px; padding: 12px; background: #f8fafc; border-radius: 10px;">
          <label style="display: flex; align-items: center; gap: 10px; cursor: pointer;">
            <input type="checkbox" bind:checked={importMerge} style="width: 18px; height: 18px;" />
            <span style="font-size: 13px; color: #475569;">合并导入（保留现有数据）</span>
          </label>
          <p style="margin: 8px 0 0 0; font-size: 12px; color: #94a3b8;">不勾选将清空现有数据后导入</p>
        </div>

        <div style="display: flex; gap: 10px;">
          <button onclick={() => showImportModal = false} style="flex: 1; padding: 12px; border-radius: 12px; font-size: 14px; font-weight: 600; cursor: pointer; border: none; background: #f1f5f9; color: #64748b;">取消</button>
          <button onclick={importData} disabled={!importJsonText.trim()} style="flex: 1; padding: 12px; border-radius: 12px; font-size: 14px; font-weight: 600; cursor: pointer; border: none; background: linear-gradient(135deg, #667eea, #764ba2); color: white; opacity: {importJsonText.trim() ? '1' : '0.5'};">导入</button>
        </div>
      </div>
    </div>
  {/if}

  <!-- 帮助弹窗 -->
  {#if showHelp}
    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;" onclick={() => showHelp = false}>
      <div style="background: white; border-radius: 20px; padding: 32px; max-width: 600px; max-height: 80vh; overflow-y: auto; box-shadow: 0 20px 60px rgba(0,0,0,0.3);" onclick={(e) => e.stopPropagation()}>
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 24px;">
          <h2 style="font-size: 24px; font-weight: 700; color: #1e293b; margin: 0;">📖 帮助文档</h2>
          <button onclick={() => showHelp = false} style="width: 36px; height: 36px; border-radius: 50%; background: #f1f5f9; border: none; cursor: pointer; font-size: 20px; color: #64748b;">×</button>
        </div>

        <div style="color: #475569; line-height: 1.8;">
          <h3 style="color: #667eea; font-size: 18px; margin: 24px 0 12px 0;">📌 内置提醒函数</h3>
          <p style="margin: 0 0 12px 0;">选择预设的提醒时间，系统会自动计算提醒时间点：</p>
          <div style="background: #f8fafc; padding: 16px; border-radius: 12px; font-size: 14px;">
            <div><strong>提前提醒：</strong>提前5分钟、10分钟、15分钟、30分钟、1小时、2小时、3小时、6小时、12小时、1天、2天、3天、1周</div>
            <div style="margin-top: 8px;"><strong>当天提醒：</strong>当天早上7/8/9点、中午12点、傍晚17/18点、晚上20点</div>
            <div style="margin-top: 8px;"><strong>隔天提醒：</strong>第二天早上8/9点</div>
          </div>

          <h3 style="color: #667eea; font-size: 18px; margin: 24px 0 12px 0;">🔧 自定义公式语法</h3>
          <p style="margin: 0 0 12px 0;">使用自定义公式可以灵活设置提醒时间：</p>

          <div style="background: #f8fafc; padding: 16px; border-radius: 12px; font-size: 14px;">
            <h4 style="margin: 0 0 12px 0; color: #1e293b;">1. 基于完成时间 (DueTime)</h4>
            <table style="width: 100%; border-collapse: collapse;">
              <tbody>
                <tr><td style="padding: 8px 0; border-bottom: 1px solid #e2e8f0;"><code style="background: #e0e7ff; padding: 2px 8px; border-radius: 4px;">DueTime-1h</code></td><td style="padding: 8px 0; border-bottom: 1px solid #e2e8f0;">完成时间前1小时</td></tr>
                <tr><td style="padding: 8px 0; border-bottom: 1px solid #e2e8f0;"><code style="background: #e0e7ff; padding: 2px 8px; border-radius: 4px;">DueTime-30m</code></td><td style="padding: 8px 0; border-bottom: 1px solid #e2e8f0;">完成时间前30分钟</td></tr>
                <tr><td style="padding: 8px 0; border-bottom: 1px solid #e2e8f0;"><code style="background: #e0e7ff; padding: 2px 8px; border-radius: 4px;">DueTime-1d</code></td><td style="padding: 8px 0; border-bottom: 1px solid #e2e8f0;">完成时间前1天</td></tr>
                <tr><td style="padding: 8px 0;"><code style="background: #e0e7ff; padding: 2px 8px; border-radius: 4px;">DueTime+1h</code></td><td style="padding: 8px 0;">完成时间后1小时</td></tr>
              </tbody>
            </table>
          </div>

          <div style="background: #f8fafc; padding: 16px; border-radius: 12px; font-size: 14px; margin-top: 16px;">
            <h4 style="margin: 0 0 12px 0; color: #1e293b;">2. 基于当天 (Date)</h4>
            <table style="width: 100%; border-collapse: collapse;">
              <tbody>
                <tr><td style="padding: 8px 0; border-bottom: 1px solid #e2e8f0;"><code style="background: #e0e7ff; padding: 2px 8px; border-radius: 4px;">Date+9h</code></td><td style="padding: 8px 0; border-bottom: 1px solid #e2e8f0;">当天早上9点</td></tr>
                <tr><td style="padding: 8px 0; border-bottom: 1px solid #e2e8f0;"><code style="background: #e0e7ff; padding: 2px 8px; border-radius: 4px;">Date+12h</code></td><td style="padding: 8px 0; border-bottom: 1px solid #e2e8f0;">当天中午12点</td></tr>
                <tr><td style="padding: 8px 0;"><code style="background: #e0e7ff; padding: 2px 8px; border-radius: 4px;">Date+18h</code></td><td style="padding: 8px 0;">当天傍晚18点</td></tr>
              </tbody>
            </table>
          </div>

          <div style="background: #fef3c7; padding: 16px; border-radius: 12px; font-size: 14px; margin-top: 16px;">
            <h4 style="margin: 0 0 8px 0; color: #92400e;">💡 单位说明</h4>
            <div><code style="background: #fde68a; padding: 2px 8px; border-radius: 4px;">m</code> = 分钟 &nbsp;&nbsp; <code style="background: #fde68a; padding: 2px 8px; border-radius: 4px;">h</code> = 小时 &nbsp;&nbsp; <code style="background: #fde68a; padding: 2px 8px; border-radius: 4px;">d</code> = 天</div>
          </div>
        </div>
      </div>
    </div>
  {/if}

  <!-- 循环任务列表弹窗 -->
  {#if showRecurringModal}
    <div style="position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; z-index: 1000;" onclick={() => showRecurringModal = false}>
      <div style="background: white; border-radius: 20px; padding: 28px; width: 700px; max-height: 80vh; overflow-y: auto; box-shadow: 0 20px 60px rgba(0,0,0,0.3);" onclick={(e) => e.stopPropagation()}>
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;">
          <h2 style="font-size: 20px; font-weight: 700; color: #1e293b; margin: 0;">🔄 循环任务管理</h2>
          <button onclick={() => showRecurringModal = false} style="width: 36px; height: 36px; border-radius: 50%; background: #f1f5f9; border: none; cursor: pointer; font-size: 20px; color: #64748b;">×</button>
        </div>

        {#if recurringTemplates.length === 0}
          <div style="text-align: center; padding: 40px; color: #94a3b8;">
            <span style="font-size: 48px;">🔄</span>
            <p style="margin-top: 12px; font-size: 15px;">暂无循环任务</p>
            <p style="font-size: 13px;">添加任务时勾选"设为循环任务"即可创建</p>
          </div>
        {:else}
          <div style="display: flex; flex-direction: column; gap: 12px;">
            {#each recurringTemplates as template}
              <div style="background: #f8fafc; border: 2px solid #e2e8f0; border-radius: 14px; padding: 16px;">
                <div style="display: flex; justify-content: space-between; align-items: flex-start;">
                  <div style="flex: 1;">
                    <div style="display: flex; align-items: center; gap: 8px; margin-bottom: 8px;">
                      <span style="font-size: 18px;">🔄</span>
                      <h3 style="font-size: 16px; font-weight: 700; color: #1e293b; margin: 0;">{template.title}</h3>
                      <span style="background: #0ea5e920; color: #0ea5e9; padding: 4px 10px; border-radius: 6px; font-size: 12px; font-weight: 600;">{getRecurrenceText(template)}</span>
                    </div>
                    {#if template.description}
                      <p style="font-size: 13px; color: #64748b; margin: 0 0 8px 0;">{template.description}</p>
                    {/if}
                    <div style="display: flex; gap: 16px; font-size: 12px; color: #64748b;">
                      <span>基准时间: {template.base_time}</span>
                      <span>已完成: {template.completed_count}次</span>
                      {#if template.next_due_time}
                        <span>下次: {formatDate(template.next_due_time)}</span>
                      {/if}
                    </div>
                  </div>
                  <div style="display: flex; gap: 8px;">
                    <button onclick={() => viewTemplateInstances(template.id)} style="padding: 8px 14px; background: #0ea5e9; color: white; border-radius: 8px; border: none; cursor: pointer; font-size: 12px; font-weight: 600;">查看记录</button>
                    <button onclick={() => deleteRecurringTemplate(template.id)} style="padding: 8px 14px; background: #ef4444; color: white; border-radius: 8px; border: none; cursor: pointer; font-size: 12px; font-weight: 600;">删除</button>
                  </div>
                </div>
              </div>
            {/each}
          </div>
        {/if}

        <!-- 循环任务实例记录 -->
        {#if selectedTemplate && templateInstances.length > 0}
          <div style="margin-top: 24px; padding-top: 24px; border-top: 2px solid #e2e8f0;">
            <h3 style="font-size: 16px; font-weight: 700; color: #1e293b; margin: 0 0 16px 0;">📋 历史记录 - {selectedTemplate.title}</h3>
            <div style="display: flex; flex-direction: column; gap: 8px; max-height: 200px; overflow-y: auto;">
              {#each templateInstances as instance}
                <div style="display: flex; justify-content: space-between; align-items: center; padding: 10px 14px; background: {instance.is_completed ? '#f0fdf4' : '#fef3c7'}; border-radius: 8px;">
                  <div style="display: flex; align-items: center; gap: 12px;">
                    <span style="font-size: 14px;">#{instance.instance_number}</span>
                    <span style="font-size: 13px; color: #64748b;">{formatDate(instance.due_time)}</span>
                  </div>
                  <span style="font-size: 12px; font-weight: 600; color: {instance.is_completed ? '#22c55e' : '#f97316'};">
                    {instance.is_completed ? '✓ 已完成' : '待完成'}
                  </span>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    </div>
  {/if}
</main>