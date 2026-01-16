import React, { createContext, useContext, useState, useCallback } from 'react';

export type ProgressType = 'upload' | 'download';

export interface ProgressTask {
    id: string;
    fileName: string;
    type: ProgressType;
    progress: number; // 0-100
    status: 'pending' | 'active' | 'completed' | 'error' | 'cancelled';
    error?: string;
    speed?: string; // e.g., "2.5 MB/s"
    cancel?: () => void;
}

interface ProgressContextType {
    tasks: ProgressTask[];
    addTask: (task: Omit<ProgressTask, 'progress' | 'status'>) => void;
    updateTask: (id: string, updates: Partial<ProgressTask>) => void;
    removeTask: (id: string) => void;
    cancelTask: (id: string) => void;
}

const ProgressContext = createContext<ProgressContextType | undefined>(undefined);

export const ProgressProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    const [tasks, setTasks] = useState<ProgressTask[]>([]);

    const addTask = useCallback((task: Omit<ProgressTask, 'progress' | 'status'>) => {
        setTasks(prev => [
            ...prev,
            { ...task, progress: 0, status: 'pending' }
        ]);
    }, []);

    const updateTask = useCallback((id: string, updates: Partial<ProgressTask>) => {
        setTasks(prev => prev.map(task => 
            task.id === id ? { ...task, ...updates } : task
        ));
    }, []);

    const removeTask = useCallback((id: string) => {
        setTasks(prev => prev.filter(task => task.id !== id));
    }, []);

    const cancelTask = useCallback((id: string) => {
        setTasks(prev => {
            const task = prev.find(t => t.id === id);
            if (task && task.cancel) {
                task.cancel();
            }
            return prev.map(t => 
                t.id === id ? { ...t, status: 'cancelled' } : t
            );
        });
    }, []);

    return (
        <ProgressContext.Provider value={{ tasks, addTask, updateTask, removeTask, cancelTask }}>
            {children}
        </ProgressContext.Provider>
    );
};

export const useProgress = () => {
    const context = useContext(ProgressContext);
    if (context === undefined) {
        throw new Error('useProgress must be used within a ProgressProvider');
    }
    return context;
};
