/**
 * Notification composable
 */

import { useNotification as useNaiveNotification } from 'naive-ui';

export function useNotification() {
  const notification = useNaiveNotification();

  const showSuccess = (message: string, title = '成功') => {
    notification.success({
      title,
      content: message,
      duration: 3000,
    });
  };

  const showError = (message: string, title = '错误') => {
    notification.error({
      title,
      content: message,
      duration: 5000,
    });
  };

  const showWarning = (message: string, title = '警告') => {
    notification.warning({
      title,
      content: message,
      duration: 4000,
    });
  };

  const showInfo = (message: string, title = '提示') => {
    notification.info({
      title,
      content: message,
      duration: 3000,
    });
  };

  return {
    showSuccess,
    showError,
    showWarning,
    showInfo,
  };
}
