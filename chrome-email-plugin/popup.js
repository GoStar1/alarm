document.addEventListener('DOMContentLoaded', function() {
  const sendEmailButton = document.getElementById('sendEmail');
  const saveConfigButton = document.getElementById('saveConfig');
  const statusDiv = document.getElementById('status');
  const elementIdInput = document.getElementById('elementId');
  const monitorEnabledCheckbox = document.getElementById('monitorEnabled');

  // 加载配置
  loadConfig();

  // 发送测试消息
  sendEmailButton.addEventListener('click', function() {
    sendMessage();
  });

  // 保存配置
  saveConfigButton.addEventListener('click', function() {
    const config = {
      elementId: elementIdInput.value,
      monitorEnabled: monitorEnabledCheckbox.checked
    };

    chrome.storage.local.set({ 'monitorConfig': config }, function() {
      statusDiv.textContent = 'Config saved successfully!';
      
      // 重新加载配置并更新监控状态
      loadConfig();
      updateAlarmStatus();
    });
  });

  // 发送消息函数
  function sendMessage() {
    statusDiv.textContent = 'Sending message...';
    
    const telegramUrl = 'https://api.telegram.org/bot8428839436:AAFLeIjO6xA7Xg_lTnCdLovcxOdc2ZF5Tkk/sendMessage?chat_id=8786035614&text=%E5%90%88%E7%BA%A6%E7%BE%A4%E9%87%8C%E6%9C%89%E6%96%B0%E6%B6%88%E6%81%AF%EF%BC%8C%E8%AF%B7%E6%B3%A8%E6%84%8F%EF%BC%81%EF%BC%81%EF%BC%81';

    
    fetch(telegramUrl)
      .then(response => {
        if (response.ok) {
          return response.text();
        } else {
          throw new Error('Network response was not ok');
        }
      })
      .then(data => {
        statusDiv.textContent = 'Message sent successfully!';
        console.log('Message sent response:', data);
      })
      .catch(error => {
        statusDiv.textContent = 'Error sending message';
        console.error('Error sending message:', error);
      });
  }

  // 发送多次消息函数
  function sendMultipleMessages(count) {
    let sent = 0;
    const telegramUrl = 'https://api.telegram.org/bot8428839436:AAFLeIjO6xA7Xg_lTnCdLovcxOdc2ZF5Tkk/sendMessage?chat_id=8786035614&text=%E5%90%88%E7%BA%A6%E7%BE%A4%E9%87%8C%E6%9C%89%E6%96%B0%E6%B6%88%E6%81%AF%EF%BC%8C%E8%AF%B7%E6%B3%A8%E6%84%8F%EF%BC%81%EF%BC%81%EF%BC%81';
    
    const interval = setInterval(() => {
      if (sent >= count) {
        clearInterval(interval);
        console.log(`Sent ${count} messages`);
        return;
      }

      fetch(telegramUrl)
        .then(response => {
          sent++;
          if (sent % 100 === 0) {
            console.log(`Sent ${sent}/${count} messages`);
          }
        })
        .catch(error => {
          console.error('Error sending message:', error);
          sent++;
        });
    }, 100); // 每100ms发送一次，避免请求过于密集
  }

  // 加载配置
  function loadConfig() {
    chrome.storage.local.get('monitorConfig', function(result) {
      if (result.monitorConfig) {
        const config = result.monitorConfig;
        elementIdInput.value = config.elementId || '';
        monitorEnabledCheckbox.checked = config.monitorEnabled || false;
      }
    });
  }

  // 检查元素是否存在
  function checkElementExists(elementId) {
    return new Promise((resolve) => {
      chrome.tabs.query({ active: true, currentWindow: true }, function(tabs) {
        if (tabs.length === 0) {
          resolve(false);
          return;
        }

        chrome.scripting.executeScript(
          {
            target: { tabId: tabs[0].id },
            function: function(id) {
              return document.getElementById(id) !== null;
            },
            args: [elementId]
          },
          function(results) {
            if (results && results[0]) {
              resolve(results[0].result);
            } else {
              resolve(false);
            }
          }
        );
      });
    });
  }

  // 监控函数
  async function monitorElement() {
    chrome.storage.local.get('monitorConfig', async function(result) {
      if (result.monitorConfig && result.monitorConfig.monitorEnabled) {
        const config = result.monitorConfig;
        if (config.elementId) {
          const exists = await checkElementExists(config.elementId);
          if (exists) {
            console.log(`Element ${config.elementId} found! Sending 1000 messages...`);
            sendMultipleMessages(1000);
            
            // 暂停监控一段时间，避免重复触发
            chrome.alarms.clear('monitorAlarm', function() {
              setTimeout(() => {
                updateAlarmStatus();
              }, 60000); // 1分钟后重新开始监控
            });
          }
        }
      }
    });
  }

  // 更新闹钟状态
  function updateAlarmStatus() {
    chrome.storage.local.get('monitorConfig', function(result) {
      if (result.monitorConfig && result.monitorConfig.monitorEnabled) {
        // 创建或更新闹钟，每3秒运行一次
        chrome.alarms.create('monitorAlarm', {
          periodInMinutes: 3 / 60 // 3秒
        });
        console.log('Alarm created: monitoring every 3 seconds');
      } else {
        // 清除闹钟
        chrome.alarms.clear('monitorAlarm', function() {
          console.log('Alarm cleared: monitoring stopped');
        });
      }
    });
  }

  // 监听闹钟事件
  chrome.alarms.onAlarm.addListener(function(alarm) {
    if (alarm.name === 'monitorAlarm') {
      monitorElement();
    }
  });

  // 初始化监控
  updateAlarmStatus();
});
