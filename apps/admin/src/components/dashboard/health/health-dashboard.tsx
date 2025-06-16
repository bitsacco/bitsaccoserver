'use client';

import * as React from 'react';
import { useState, useEffect } from 'react';
import Box from '@mui/material/Box';
import Card from '@mui/material/Card';
import CardContent from '@mui/material/CardContent';
import CardHeader from '@mui/material/CardHeader';
import Container from '@mui/material/Container';
import Grid from '@mui/material/Unstable_Grid2';
import Stack from '@mui/material/Stack';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import CircularProgress from '@mui/material/CircularProgress';
import Alert from '@mui/material/Alert';
import Chip from '@mui/material/Chip';
import List from '@mui/material/List';
import ListItem from '@mui/material/ListItem';
import ListItemIcon from '@mui/material/ListItemIcon';
import ListItemText from '@mui/material/ListItemText';
import { ArrowClockwise as ArrowClockwiseIcon } from '@phosphor-icons/react/dist/ssr/ArrowClockwise';
import { CheckCircle as CheckCircleIcon } from '@phosphor-icons/react/dist/ssr/CheckCircle';
import { Warning as WarningIcon } from '@phosphor-icons/react/dist/ssr/Warning';
import { XCircle as XCircleIcon } from '@phosphor-icons/react/dist/ssr/XCircle';

import { fetchSystemHealth } from '@/lib/metrics/client';
import type { SystemHealthData } from '@/types/metrics';

export function HealthDashboard(): React.JSX.Element {
  const [healthData, setHealthData] = useState<SystemHealthData | null>(null);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<Error | null>(null);

  const fetchData = async () => {
    try {
      setIsLoading(true);
      setError(null);
      const data = await fetchSystemHealth();
      setHealthData(data);
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchData();
  }, []);

  const getStatusColor = (status: 'healthy' | 'degraded' | 'down') => {
    switch (status) {
      case 'healthy':
        return 'success';
      case 'degraded':
        return 'warning';
      case 'down':
        return 'error';
      default:
        return 'default';
    }
  };

  const getStatusIcon = (status: 'healthy' | 'degraded' | 'down') => {
    switch (status) {
      case 'healthy':
        return <CheckCircleIcon color="green" />;
      case 'degraded':
        return <WarningIcon color="orange" />;
      case 'down':
        return <XCircleIcon color="red" />;
      default:
        return null;
    }
  };

  return (
    <Box component="main" sx={{ flexGrow: 1, py: 8 }}>
      <Container maxWidth="xl">
        <Stack spacing={3}>
          <Stack direction="row" justifyContent="space-between" spacing={4}>
            <Stack spacing={1}>
              <Typography variant="h4">System Health</Typography>
              <Stack alignItems="center" direction="row" spacing={1}>
                <Typography color="text.secondary" variant="body2">
                  Last updated:{' '}
                  {healthData?.lastChecked
                    ? new Date(healthData.lastChecked).toLocaleString()
                    : 'Never'}
                </Typography>
              </Stack>
            </Stack>
            <Button
              startIcon={
                <ArrowClockwiseIcon fontSize="var(--icon-fontSize-md)" />
              }
              variant="contained"
              onClick={fetchData}
              disabled={isLoading}
            >
              Refresh
            </Button>
          </Stack>

          {/* Loading state */}
          {isLoading && (
            <Box sx={{ display: 'flex', justifyContent: 'center', mt: 4 }}>
              <CircularProgress />
            </Box>
          )}

          {/* Error state */}
          {error && (
            <Alert severity="error" sx={{ mt: 2 }}>
              Error loading system health: {error.message}
            </Alert>
          )}

          {/* Health data */}
          {healthData && !isLoading && (
            <React.Fragment>
              {/* Overall Status */}
              <Card>
                <CardHeader title="Overall System Status" />
                <CardContent>
                  <Stack direction="row" alignItems="center" spacing={2}>
                    {getStatusIcon(healthData.overall)}
                    <Chip
                      label={healthData.overall.toUpperCase()}
                      color={getStatusColor(healthData.overall) as any}
                      variant="filled"
                    />
                    <Typography variant="body1">
                      System is{' '}
                      {healthData.overall === 'healthy'
                        ? 'operating normally'
                        : healthData.overall === 'degraded'
                          ? 'experiencing some issues'
                          : 'experiencing critical problems'}
                    </Typography>
                  </Stack>
                </CardContent>
              </Card>

              <Grid container spacing={3}>
                {/* Server Status */}
                <Grid xs={12} md={6}>
                  <Card sx={{ height: '100%' }}>
                    <CardHeader title="Server Status" />
                    <CardContent>
                      {healthData.server ? (
                        <Stack spacing={2}>
                          <Typography variant="body2">
                            <strong>Uptime:</strong>{' '}
                            {healthData.server.uptime || 'Unknown'}
                          </Typography>
                          <Typography variant="body2">
                            <strong>Memory Usage:</strong>{' '}
                            {healthData.server.memory
                              ? typeof healthData.server.memory === 'object'
                                ? `${Math.round(healthData.server.memory.heapUsed / 1024 / 1024)}MB / ${Math.round(healthData.server.memory.heapTotal / 1024 / 1024)}MB`
                                : healthData.server.memory
                              : 'Unknown'}
                          </Typography>
                          <Typography variant="body2">
                            <strong>CPU Usage:</strong>{' '}
                            {healthData.server.cpu
                              ? typeof healthData.server.cpu === 'object'
                                ? `${Math.round((healthData.server.cpu.user + healthData.server.cpu.system) * 100)}%`
                                : healthData.server.cpu
                              : 'Unknown'}
                          </Typography>
                          <Typography variant="body2">
                            <strong>Version:</strong>{' '}
                            {healthData.server.version || 'Unknown'}
                          </Typography>
                        </Stack>
                      ) : (
                        <Typography variant="body2" color="text.secondary">
                          No server information available
                        </Typography>
                      )}
                    </CardContent>
                  </Card>
                </Grid>

                {/* Services Status */}
                <Grid xs={12} md={6}>
                  <Card sx={{ height: '100%' }}>
                    <CardHeader title="Services Status" />
                    <CardContent>
                      {healthData.services && healthData.services.length > 0 ? (
                        <List dense>
                          {healthData.services.map(
                            (service: any, index: number) => (
                              <ListItem key={index}>
                                <ListItemIcon>
                                  {getStatusIcon(service.status || 'healthy')}
                                </ListItemIcon>
                                <ListItemText
                                  primary={
                                    service.name || `Service ${index + 1}`
                                  }
                                  secondary={service.status || 'healthy'}
                                />
                              </ListItem>
                            ),
                          )}
                        </List>
                      ) : (
                        <Typography variant="body2" color="text.secondary">
                          No service information available
                        </Typography>
                      )}
                    </CardContent>
                  </Card>
                </Grid>

                {/* Integrations Status */}
                <Grid xs={12}>
                  <Card>
                    <CardHeader title="Integrations Status" />
                    <CardContent>
                      {healthData.integrations &&
                      healthData.integrations.length > 0 ? (
                        <Grid container spacing={2}>
                          {healthData.integrations.map(
                            (integration: any, index: number) => (
                              <Grid xs={12} sm={6} md={4} key={index}>
                                <Card variant="outlined">
                                  <CardContent>
                                    <Stack
                                      direction="row"
                                      alignItems="center"
                                      spacing={1}
                                    >
                                      {getStatusIcon(
                                        integration.status || 'healthy',
                                      )}
                                      <Stack>
                                        <Typography variant="subtitle2">
                                          {integration.name ||
                                            `Integration ${index + 1}`}
                                        </Typography>
                                        <Typography
                                          variant="body2"
                                          color="text.secondary"
                                        >
                                          {integration.type || 'Unknown type'}
                                        </Typography>
                                        <Chip
                                          size="small"
                                          label={
                                            integration.status || 'healthy'
                                          }
                                          color={
                                            getStatusColor(
                                              integration.status || 'healthy',
                                            ) as any
                                          }
                                        />
                                      </Stack>
                                    </Stack>
                                  </CardContent>
                                </Card>
                              </Grid>
                            ),
                          )}
                        </Grid>
                      ) : (
                        <Typography variant="body2" color="text.secondary">
                          No integration information available
                        </Typography>
                      )}
                    </CardContent>
                  </Card>
                </Grid>
              </Grid>
            </React.Fragment>
          )}
        </Stack>
      </Container>
    </Box>
  );
}
