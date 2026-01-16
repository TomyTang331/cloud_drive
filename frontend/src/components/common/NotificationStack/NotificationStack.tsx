import React from 'react';
import { useToast } from '../../../context/ToastContext';
import { useProgress } from '../../../context/ProgressContext';
import Toast from '../Toast/Toast';
import { ProgressItem } from '../Progress/Progress';
import './NotificationStack.less';

const NotificationStack: React.FC = () => {
    const { toasts, removeToast } = useToast();
    const { tasks } = useProgress();

    // Only render if there are items
    if (toasts.length === 0 && tasks.length === 0) return null;

    return (
        <div className="notification-stack">
            {/* Tasks (Progress) */}
            {tasks.map(task => (
                <div key={task.id} className="notification-wrapper">
                    <ProgressItem task={task} />
                </div>
            ))}

            {/* Toasts */}
            {toasts.map(toast => (
                <div key={toast.id} className="notification-wrapper">
                    <Toast {...toast} onClose={removeToast} />
                </div>
            ))}
        </div>
    );
};

export default NotificationStack;
