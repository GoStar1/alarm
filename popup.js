document.addEventListener('DOMContentLoaded', function() {
  const sendBtn = document.getElementById('sendBtn');
  const statusDiv = document.getElementById('status');

  function showStatus(message, isSuccess) {
    statusDiv.textContent = message;
    statusDiv.className = isSuccess ? 'success' : 'error';
    statusDiv.style.display = 'block';
  }

  function loadConfig() {
    chrome.storage.local.get(['smtpServer', 'smtpPort', 'username', 'password', 'toEmail'], function(result) {
      if (result.smtpServer) document.getElementById('smtpServer').value = result.smtpServer;
      if (result.smtpPort) document.getElementById('smtpPort').value = result.smtpPort;
      if (result.username) document.getElementById('username').value = result.username;
      if (result.password) document.getElementById('password').value = result.password;
      if (result.toEmail) document.getElementById('toEmail').value = result.toEmail;
    });
  }

  function saveConfig() {
    const config = {
      smtpServer: document.getElementById('smtpServer').value,
      smtpPort: document.getElementById('smtpPort').value,
      username: document.getElementById('username').value,
      password: document.getElementById('password').value,
      toEmail: document.getElementById('toEmail').value
    };
    chrome.storage.local.set(config);
  }

  sendBtn.addEventListener('click', async function() {
    const smtpServer = document.getElementById('smtpServer').value;
    const smtpPort = document.getElementById('smtpPort').value;
    const username = document.getElementById('username').value;
    const password = document.getElementById('password').value;
    const toEmail = document.getElementById('toEmail').value;
    const subject = document.getElementById('subject').value;
    const body = document.getElementById('body').value;

    if (!smtpServer || !smtpPort || !username || !password || !toEmail || !subject || !body) {
      showStatus('请填写所有字段', false);
      return;
    }

    saveConfig();
    sendBtn.disabled = true;
    sendBtn.textContent = '发送中...';

    try {
      const emailData = {
        Host: smtpServer,
        Username: username,
        Password: password,
        To: toEmail,
        From: username,
        Subject: subject,
        Body: body
      };

      const response = await fetch('https://smtpjs.com/v3/smtpjs.aspx', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/x-www-form-urlencoded'
        },
        body: new URLSearchParams({
          action: 'Send',
          data: JSON.stringify(emailData)
        })
      });

      const result = await response.text();
      
      if (result.includes('OK')) {
        showStatus('邮件发送成功!', true);
      } else {
        showStatus('发送失败: ' + result, false);
      }
    } catch (error) {
      showStatus('发送失败: ' + error.message, false);
    } finally {
      sendBtn.disabled = false;
      sendBtn.textContent = '发送邮件';
    }
  });

  loadConfig();
});
