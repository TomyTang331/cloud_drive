import React, { createContext, useContext, useState, useEffect } from 'react';
import type { ReactNode } from 'react';
import { authService } from '../services/api';
import type { AuthResponse } from '../types';

interface AuthContextType {
    user: AuthResponse | null;
    login: (username: string, password: string, rememberMe?: boolean) => Promise<void>;
    register: (username: string, email: string, password: string) => Promise<void>;
    logout: () => void;
    isAuthenticated: boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const AuthProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    const [user, setUser] = useState<AuthResponse | null>(null);

    useEffect(() => {
        const token = localStorage.getItem('token');
        const savedUser = localStorage.getItem('user');
        const expiresAt = localStorage.getItem('token_expires_at');

        if (token && savedUser) {
            // Check if token has expired
            if (expiresAt) {
                const expirationTime = parseInt(expiresAt, 10);
                if (Date.now() > expirationTime) {
                    // Token expired, clear everything
                    localStorage.removeItem('token');
                    localStorage.removeItem('user');
                    localStorage.removeItem('token_expires_at');
                    return;
                }
            }
            setUser(JSON.parse(savedUser));
        }
    }, []);

    const login = async (username: string, password: string, rememberMe: boolean = false) => {
        const response = await authService.login({ username, password });
        const userData = response.data.data;
        setUser(userData);
        localStorage.setItem('token', userData.token);
        localStorage.setItem('user', JSON.stringify(userData));

        // Set expiration time if "Remember me" is checked
        if (rememberMe) {
            const expiresAt = Date.now() + (30 * 24 * 60 * 60 * 1000); // 30 days
            localStorage.setItem('token_expires_at', expiresAt.toString());
        } else {
            // If not remembering, remove any existing expiration
            localStorage.removeItem('token_expires_at');
        }
    };

    const register = async (username: string, email: string, password: string) => {
        const response = await authService.register({ username, email, password });
        const userData = response.data.data;
        setUser(userData);
        localStorage.setItem('token', userData.token);
        localStorage.setItem('user', JSON.stringify(userData));
    };

    const logout = () => {
        setUser(null);
        localStorage.removeItem('token');
        localStorage.removeItem('user');
        localStorage.removeItem('token_expires_at');
    };

    return (
        <AuthContext.Provider value={{ user, login, register, logout, isAuthenticated: !!user }}>
            {children}
        </AuthContext.Provider>
    );
};

export const useAuth = () => {
    const context = useContext(AuthContext);
    if (context === undefined) {
        throw new Error('useAuth must be used within an AuthProvider');
    }
    return context;
};
