import React from 'react';
import Toast, { type ToastProps } from './Toast';
import './Toast.less';

interface ToastContainerProps {
    toasts: ToastProps[];
    onClose: (id: string) => void;
}

const ToastContainer: React.FC<ToastContainerProps> = ({ toasts, onClose }) => {
    return (
        <div className="toast-container">
            {toasts.map((toast) => (
                <Toast key={toast.id} {...toast} onClose={onClose} />
            ))}
        </div>
    );
};

export default ToastContainer;
