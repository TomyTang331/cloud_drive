import api from './client';
import type { ApiResponse, AuthResponse, LoginData, RegisterData, UserProfile } from '../../types';

export const authService = {
    register: (data: RegisterData) =>
        api.post<ApiResponse<AuthResponse>>('/api/auth/register', data),

    login: (data: LoginData) =>
        api.post<ApiResponse<AuthResponse>>('/api/auth/login', data),

    getProfile: () =>
        api.get<ApiResponse<UserProfile>>('/api/users/profile'),
};
