'use client';

import * as React from 'react';
import { useState, useEffect } from 'react';
import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import Container from '@mui/material/Container';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';
import Tab from '@mui/material/Tab';
import Tabs from '@mui/material/Tabs';
import CircularProgress from '@mui/material/CircularProgress';
import Alert from '@mui/material/Alert';

import { fetchWithAuth } from '@/lib/fetch-with-auth';
import { useUser } from '@/hooks/use-user';
import { ServiceRole } from '@bitsaccoserver/types';

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props;

  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      id={`admin-tabpanel-${index}`}
      aria-labelledby={`admin-tab-${index}`}
      {...other}
    >
      {value === index && <Box sx={{ p: 3 }}>{children}</Box>}
    </div>
  );
}

export function AdminSettingsDashboard(): React.JSX.Element {
  const [tabValue, setTabValue] = useState(0);
  const [configData, setConfigData] = useState<any>(null);
  const [integrationsData, setIntegrationsData] = useState<any>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<Error | null>(null);

  const { user } = useUser();
  const isSystemAdmin = user?.serviceRole === ServiceRole.SYSTEM_ADMIN;

  const fetchAdminData = async () => {
    try {
      setIsLoading(true);
      setError(null);

      // Fetch system configuration (SYSTEM_ADMIN only)
      if (isSystemAdmin) {
        try {
          const configResponse = await fetchWithAuth('/admin/config');
          if (configResponse.ok) {
            const config = await configResponse.json();
            setConfigData(config);
          }
        } catch (configError) {
          console.warn('Could not fetch config data:', configError);
        }
      }

      // Fetch integrations (ADMIN+)
      try {
        const integrationsResponse = await fetchWithAuth('/admin/integrations');
        if (integrationsResponse.ok) {
          const integrations = await integrationsResponse.json();
          setIntegrationsData(integrations);
        }
      } catch (integrationsError) {
        console.warn('Could not fetch integrations data:', integrationsError);
      }
    } catch (err) {
      setError(
        err instanceof Error ? err : new Error('Failed to fetch admin data'),
      );
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchAdminData();
  }, [isSystemAdmin]);

  const handleTabChange = (event: React.SyntheticEvent, newValue: number) => {
    setTabValue(newValue);
  };

  const renderConfigData = () => {
    if (!isSystemAdmin) {
      return (
        <Alert severity="warning">
          System Configuration access requires SYSTEM_ADMIN role
        </Alert>
      );
    }

    if (!configData) {
      return (
        <Typography variant="body2" color="text.secondary">
          No configuration data available
        </Typography>
      );
    }

    return (
      <Box
        component="pre"
        sx={{
          backgroundColor: 'grey.100',
          p: 2,
          borderRadius: 1,
          overflow: 'auto',
          maxHeight: 600,
          fontSize: '0.875rem',
          fontFamily: 'monospace',
        }}
      >
        {JSON.stringify(configData, null, 2)}
      </Box>
    );
  };

  const renderIntegrationsData = () => {
    if (!integrationsData) {
      return (
        <Typography variant="body2" color="text.secondary">
          No integrations data available
        </Typography>
      );
    }

    return (
      <Stack spacing={2}>
        {Array.isArray(integrationsData) ? (
          integrationsData.map((integration: any, index: number) => (
            <Card key={index} variant="outlined">
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  {integration.serviceName || `Integration ${index + 1}`}
                </Typography>
                <Typography variant="body2" color="text.secondary" gutterBottom>
                  Type: {integration.serviceType || 'Unknown'}
                </Typography>
                <Typography variant="body2" color="text.secondary" gutterBottom>
                  Provider: {integration.provider || 'Unknown'}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  Status: {integration.isEnabled ? 'Enabled' : 'Disabled'}
                </Typography>
              </CardContent>
            </Card>
          ))
        ) : (
          <Box
            component="pre"
            sx={{
              backgroundColor: 'grey.100',
              p: 2,
              borderRadius: 1,
              overflow: 'auto',
              maxHeight: 600,
              fontSize: '0.875rem',
              fontFamily: 'monospace',
            }}
          >
            {JSON.stringify(integrationsData, null, 2)}
          </Box>
        )}
      </Stack>
    );
  };

  return (
    <Box component="main" sx={{ flexGrow: 1, py: 8 }}>
      <Container maxWidth="xl">
        <Stack spacing={3}>
          <Stack spacing={1}>
            <Typography variant="h4">Admin Settings</Typography>
            <Typography color="text.secondary" variant="body2">
              Manage system configuration and integrations
            </Typography>
          </Stack>

          {/* Loading state */}
          {isLoading && (
            <Box sx={{ display: 'flex', justifyContent: 'center', mt: 4 }}>
              <CircularProgress />
            </Box>
          )}

          {/* Error state */}
          {error && (
            <Alert severity="error">
              Error loading admin settings: {error.message}
            </Alert>
          )}

          {/* Settings content */}
          {!isLoading && (
            <Card>
              <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
                <Tabs value={tabValue} onChange={handleTabChange}>
                  <Tab label="Integrations" />
                  {isSystemAdmin && <Tab label="System Configuration" />}
                </Tabs>
              </Box>

              <TabPanel value={tabValue} index={0}>
                <Stack spacing={2}>
                  <Typography variant="h6">Service Integrations</Typography>
                  <Typography variant="body2" color="text.secondary">
                    View and manage external service integrations
                  </Typography>
                  {renderIntegrationsData()}
                </Stack>
              </TabPanel>

              {isSystemAdmin && (
                <TabPanel value={tabValue} index={1}>
                  <Stack spacing={2}>
                    <Typography variant="h6">System Configuration</Typography>
                    <Typography variant="body2" color="text.secondary">
                      View system-level configuration settings (SYSTEM_ADMIN
                      only)
                    </Typography>
                    {renderConfigData()}
                  </Stack>
                </TabPanel>
              )}
            </Card>
          )}
        </Stack>
      </Container>
    </Box>
  );
}
