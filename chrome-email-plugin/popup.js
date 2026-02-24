document.addEventListener('DOMContentLoaded', function() {
  const sendEmailButton = document.getElementById('sendEmail');
  const statusDiv = document.getElementById('status');

  sendEmailButton.addEventListener('click', function() {
    statusDiv.textContent = 'Sending email...';
    
    // 发送 GET 请求到本地服务器
    fetch('http://localhost:8888/send-email')
      .then(response => {
        if (response.ok) {
          return response.text();
        } else {
          throw new Error('Network response was not ok');
        }
      })
      .then(data => {
        statusDiv.textContent = 'Email sent successfully!';
        console.log('Email sent response:', data);
      })
      .catch(error => {
        statusDiv.textContent = 'Error sending email';
        console.error('Error sending email:', error);
      });
  });
});
