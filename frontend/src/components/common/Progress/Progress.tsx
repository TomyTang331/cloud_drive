import React from 'react';
import { useProgress, type ProgressTask } from '../../../context/ProgressContext';
import './Progress.less';

// Inline Icons to avoid dependency issues
const Icons = {
    Upload: () => (
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
            <polyline points="17 8 12 3 7 8"></polyline>
            <line x1="12" y1="3" x2="12" y2="15"></line>
        </svg>
    ),
    Download: () => (
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path>
            <polyline points="7 10 12 15 17 10"></polyline>
            <line x1="12" y1="15" x2="12" y2="3"></line>
        </svg>
    ),
    X: () => (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <line x1="18" y1="6" x2="6" y2="18"></line>
            <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
    ),
    CheckCircle: () => (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path>
            <polyline points="22 4 12 14.01 9 11.01"></polyline>
        </svg>
    ),
    AlertCircle: () => (
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
            <circle cx="12" cy="12" r="10"></circle>
            <line x1="12" y1="8" x2="12" y2="12"></line>
            <line x1="12" y1="16" x2="12.01" y2="16"></line>
        </svg>
    )
};

export const ProgressItem: React.FC<{ task: ProgressTask }> = ({ task }) => {
    const { removeTask, cancelTask } = useProgress();

    // Auto remove completed tasks after 3 seconds
    React.useEffect(() => {
        if (task.status === 'completed' || task.status === 'error' || task.status === 'cancelled') {
            const timer = setTimeout(() => {
                removeTask(task.id);
            }, 3000);
            return () => clearTimeout(timer);
        }
    }, [task.status, task.id, removeTask]);

    const isUpload = task.type === 'upload';
    const isError = task.status === 'error';
    const isCompleted = task.status === 'completed';

    return (
        <div className={`progress-item ${isCompleted ? 'completed' : ''}`}>
            <div className="progress-header">
                <div className="progress-title">
                    <div className={`progress-icon ${!isUpload ? 'download' : ''}`}>
                        {isUpload ? <Icons.Upload /> : <Icons.Download />}
                    </div>
                    <span title={task.fileName}>{task.fileName}</span>
                </div>
                <div className="progress-actions">
                    {task.status === 'active' || task.status === 'pending' ? (
                        <button className="progress-cancel" onClick={() => cancelTask(task.id)}>
                            <Icons.X />
                        </button>
                    ) : isCompleted ? (
                        <span style={{ color: '#388E3C', display: 'flex' }}><Icons.CheckCircle /></span>
                    ) : isError ? (
                        <span style={{ color: '#D32F2F', display: 'flex' }}><Icons.AlertCircle /></span>
                    ) : null}
                </div>
            </div>

            <div className="progress-bar-bg">
                <div
                    className={`progress-bar-fill ${!isUpload ? 'download' : ''} ${isError ? 'error' : ''}`}
                    style={{ width: `${task.progress}%` }}
                />
            </div>

            <div className="progress-header" style={{ marginTop: '8px', marginBottom: 0 }}>
                <div className="progress-info">
                    {task.status === 'error' ? (
                        <span style={{ color: '#D32F2F' }}>{task.error || 'Failed'}</span>
                    ) : task.status === 'cancelled' ? (
                        <span>Cancelled</span>
                    ) : task.status === 'completed' ? (
                        <span style={{ color: '#388E3C' }}>Completed</span>
                    ) : (
                        <>
                            <span>{Math.round(task.progress)}%</span>
                            {task.speed && <span>• {task.speed}</span>}
                        </>
                    )}
                </div>
            </div>
        </div>
    );
};

export const ProgressContainer: React.FC = () => {
    const { tasks } = useProgress();

    if (tasks.length === 0) return null;

    return (
        <div className="progress-container">
            {tasks.map(task => (
                <ProgressItem key={task.id} task={task} />
            ))}
        </div>
    );
};
