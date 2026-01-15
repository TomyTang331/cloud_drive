import React, { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import './Auth.less';

const Login: React.FC = () => {
    const [username, setUsername] = useState('');
    const [password, setPassword] = useState('');
    const [showPassword, setShowPassword] = useState(false);
    const [rememberMe, setRememberMe] = useState(false);
    const [error, setError] = useState('');
    const [loading, setLoading] = useState(false);
    const { login } = useAuth();
    const navigate = useNavigate();

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError('');
        setLoading(true);

        try {
            await login(username, password, rememberMe);
            navigate('/dashboard');
        } catch (err: any) {
            setError(err.response?.data?.error || 'Login failed. Please check your credentials.');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="auth-container">
            <div className="auth-left">
                <div className="illustration">
                    <svg viewBox="0 0 300 300" className="illustration-svg">
                        <circle cx="100" cy="200" r="60" fill="#FF8C42" />
                        <rect x="140" y="140" width="70" height="100" rx="10" fill="#6B4FFF" />
                        <rect x="180" y="170" width="40" height="50" rx="5" fill="#1A1A1A" />
                        <rect x="220" y="160" width="60" height="80" rx="10" fill="#FFD93D" />
                        <line x1="240" y1="190" x2="260" y2="190" stroke="#1A1A1A" strokeWidth="3" />
                    </svg>
                </div>
            </div>

            <div className="auth-right">
                <div className="auth-form-container">
                    <div className="logo">★</div>

                    <h1>Welcome back!</h1>
                    <p className="subtitle">Please enter your details</p>

                    {error && <div className="error-message">{error}</div>}

                    <form onSubmit={handleSubmit}>
                        <div className="form-group">
                            <input
                                type="text"
                                placeholder="Username"
                                value={username}
                                onChange={(e) => setUsername(e.target.value)}
                                required
                            />
                        </div>

                        <div className="form-group password-group">
                            <input
                                type={showPassword ? 'text' : 'password'}
                                placeholder="Password"
                                value={password}
                                onChange={(e) => setPassword(e.target.value)}
                                required
                            />
                            <button
                                type="button"
                                className="toggle-password"
                                onClick={() => setShowPassword(!showPassword)}
                            >
                                {showPassword ? '👁️' : '👁️‍🗨️'}
                            </button>
                        </div>

                        <div className="form-footer">
                            <label className="checkbox-label">
                                <input
                                    type="checkbox"
                                    checked={rememberMe}
                                    onChange={(e) => setRememberMe(e.target.checked)}
                                />
                                Remember for 30 days
                            </label>
                            <Link to="/forgot-password" className="forgot-password">Forgot password?</Link>
                        </div>

                        <button type="submit" className="btn-primary" disabled={loading}>
                            {loading ? 'Logging in...' : 'Log In'}
                        </button>

                        {/* Temporarily disabled Google Login
                        <button type="button" className="btn-google" disabled>
                            <svg width="18" height="18" viewBox="0 0 18 18">
                                <path fill="#4285F4" d="M17.64,9.2c0-0.64-0.06-1.25-0.16-1.84H9v3.48h4.84c-0.21,1.12-0.84,2.07-1.8,2.71v2.26h2.92C16.66,14.09,17.64,11.85,17.64,9.2z" />
                                <path fill="#34A853" d="M9,18c2.43,0,4.47-0.81,5.96-2.19l-2.92-2.26c-0.81,0.54-1.84,0.86-3.04,0.86c-2.34,0-4.32-1.58-5.03-3.71H1v2.33C2.48,15.98,5.48,18,9,18z" />
                                <path fill="#FBBC05" d="M3.97,10.7c-0.18-0.54-0.28-1.11-0.28-1.7s0.1-1.16,0.28-1.7V4.97H1C0.36,6.24,0,7.59,0,9s0.36,2.76,1,4.03L3.97,10.7z" />
                                <path fill="#EA4335" d="M9,3.58c1.32,0,2.5,0.45,3.44,1.35l2.58-2.58C13.46,0.89,11.43,0,9,0C5.48,0,2.48,2.02,1,4.97l2.97,2.33C4.68,5.16,6.66,3.58,9,3.58z" />
                            </svg>
                            Log In with Google
                        </button>
                        */}
                    </form>

                    <p className="signup-link">
                        Don't have an account? <Link to="/signup">Sign Up</Link>
                    </p>
                </div>
            </div>
        </div>
    );
};

export default Login;
