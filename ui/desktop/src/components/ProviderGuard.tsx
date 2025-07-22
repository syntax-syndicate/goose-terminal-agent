import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useConfig } from './ConfigContext';
import { SetupModal } from './SetupModal';
import { startOpenRouterSetup } from '../utils/openRouterSetup';

interface ProviderGuardProps {
  children: React.ReactNode;
}

export default function ProviderGuard({ children }: ProviderGuardProps) {
  const { read } = useConfig();
  const navigate = useNavigate();
  const [isChecking, setIsChecking] = useState(true);
  const [hasProvider, setHasProvider] = useState(false);
  const [isSettingUp, setIsSettingUp] = useState(false);
  const [openRouterSetupState, setOpenRouterSetupState] = useState<{
    show: boolean;
    title: string;
    message: string;
    showProgress: boolean;
    showRetry: boolean;
    autoClose?: number;
  } | null>(null);

  const handleOpenRouterSetup = async () => {
    setOpenRouterSetupState({
      show: true,
      title: 'Setting up OpenRouter',
      message: 'A browser window will open for authentication...',
      showProgress: true,
      showRetry: false,
    });

    try {
      const result = await startOpenRouterSetup();

      if (result.success) {
        setOpenRouterSetupState({
          show: true,
          title: 'Setup Complete!',
          message: 'OpenRouter has been configured successfully.',
          showProgress: false,
          showRetry: false,
          autoClose: 3000,
        });

        // Reload the page after successful setup
        setTimeout(() => {
          window.location.reload();
        }, 3000);
      } else {
        setOpenRouterSetupState({
          show: true,
          title: 'Setup Pending',
          message: result.message,
          showProgress: false,
          showRetry: true,
        });
      }
    } catch (error) {
      setOpenRouterSetupState({
        show: true,
        title: 'Setup Error',
        message: 'Failed to complete OpenRouter setup',
        showProgress: false,
        showRetry: true,
      });
    }
  };

  useEffect(() => {
    const checkProvider = async () => {
      try {
        const config = window.electron.getConfig();
        console.log('ProviderGuard - Full config:', config);
        console.log('ProviderGuard - GOOSE_STARTUP:', config.GOOSE_STARTUP);

        const provider = (await read('GOOSE_PROVIDER', false)) ?? config.GOOSE_DEFAULT_PROVIDER;
        const model = (await read('GOOSE_MODEL', false)) ?? config.GOOSE_DEFAULT_MODEL;

        console.log('ProviderGuard - Provider:', provider, 'Model:', model);

        if (provider && model) {
          console.log('ProviderGuard - Provider and model found, continuing normally');
          setHasProvider(true);
        } else {
          console.log('ProviderGuard - No provider/model configured');
          // Check if GOOSE_STARTUP=openrouter
          const startupMode = config.GOOSE_STARTUP;
          if (startupMode === 'openrouter' && !isSettingUp) {
            console.log('GOOSE_STARTUP=openrouter detected, starting OpenRouter setup');
            setIsSettingUp(true);
            // Start OpenRouter setup automatically
            handleOpenRouterSetup();
          } else if (!isSettingUp) {
            console.log(
              'No provider/model configured, redirecting to welcome. GOOSE_STARTUP:',
              startupMode
            );
            navigate('/welcome', { replace: true });
          }
        }
      } catch (error) {
        console.error('Error checking provider configuration:', error);
        // On error, assume no provider and redirect to welcome
        navigate('/welcome', { replace: true });
      } finally {
        setIsChecking(false);
      }
    };

    checkProvider();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [read, navigate]);

  if (isChecking && !openRouterSetupState?.show) {
    return (
      <div className="flex justify-center items-center py-12">
        <div className="animate-spin rounded-full h-8 w-8 border-t-2 border-b-2 border-textStandard"></div>
      </div>
    );
  }

  if (openRouterSetupState?.show) {
    return (
      <SetupModal
        title={openRouterSetupState.title}
        message={openRouterSetupState.message}
        showProgress={openRouterSetupState.showProgress}
        showRetry={openRouterSetupState.showRetry}
        onRetry={handleOpenRouterSetup}
        autoClose={openRouterSetupState.autoClose}
        onClose={() => setOpenRouterSetupState(null)}
      />
    );
  }

  if (!hasProvider) {
    // This will be handled by the navigation above, but we return null to be safe
    return null;
  }

  return <>{children}</>;
}
