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
import TextField from '@mui/material/TextField';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import { ArrowClockwise as ArrowClockwiseIcon } from '@phosphor-icons/react/dist/ssr/ArrowClockwise';

import {
  fetchOrganizationMetrics,
  fetchChamaMetrics,
} from '@/lib/metrics/client';

export function AdminMetricsDashboard(): React.JSX.Element {
  const [metricType, setMetricType] = useState<'organization' | 'chama'>(
    'organization',
  );
  const [entityId, setEntityId] = useState<string>('');
  const [timeRange, setTimeRange] = useState<'1h' | '24h' | '7d' | '30d'>(
    '24h',
  );
  const [metricsData, setMetricsData] = useState<any>(null);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<Error | null>(null);

  const fetchMetrics = async () => {
    if (!entityId.trim()) {
      setError(new Error('Please enter an entity ID'));
      return;
    }

    try {
      setIsLoading(true);
      setError(null);

      let data;
      if (metricType === 'organization') {
        data = await fetchOrganizationMetrics(entityId, timeRange);
      } else {
        data = await fetchChamaMetrics(entityId, timeRange);
      }

      setMetricsData(data);
    } catch (err) {
      setError(
        err instanceof Error ? err : new Error('Failed to fetch metrics'),
      );
    } finally {
      setIsLoading(false);
    }
  };

  const renderMetricCard = (title: string, value: any) => (
    <Card variant="outlined">
      <CardContent>
        <Typography variant="h6" gutterBottom>
          {title}
        </Typography>
        <Typography variant="body2" color="text.secondary">
          {typeof value === 'object'
            ? JSON.stringify(value, null, 2)
            : String(value || 'No data')}
        </Typography>
      </CardContent>
    </Card>
  );

  return (
    <Box component="main" sx={{ flexGrow: 1, py: 8 }}>
      <Container maxWidth="xl">
        <Stack spacing={3}>
          <Stack spacing={1}>
            <Typography variant="h4">Admin Metrics Dashboard</Typography>
            <Typography color="text.secondary" variant="body2">
              View detailed metrics for organizations and chamas
            </Typography>
          </Stack>

          {/* Controls */}
          <Card>
            <CardContent>
              <Grid container spacing={2} alignItems="center">
                <Grid xs={12} sm={6} md={3}>
                  <FormControl fullWidth size="small">
                    <InputLabel>Type</InputLabel>
                    <Select
                      value={metricType}
                      onChange={(e) =>
                        setMetricType(
                          e.target.value as 'organization' | 'chama',
                        )
                      }
                      label="Type"
                    >
                      <MenuItem value="organization">Organization</MenuItem>
                      <MenuItem value="chama">Chama</MenuItem>
                    </Select>
                  </FormControl>
                </Grid>

                <Grid xs={12} sm={6} md={3}>
                  <TextField
                    fullWidth
                    size="small"
                    label={`${metricType === 'organization' ? 'Organization' : 'Chama'} ID`}
                    value={entityId}
                    onChange={(e) => setEntityId(e.target.value)}
                    placeholder="Enter ID..."
                  />
                </Grid>

                <Grid xs={12} sm={6} md={3}>
                  <FormControl fullWidth size="small">
                    <InputLabel>Time Range</InputLabel>
                    <Select
                      value={timeRange}
                      onChange={(e) => setTimeRange(e.target.value as any)}
                      label="Time Range"
                    >
                      <MenuItem value="1h">Last Hour</MenuItem>
                      <MenuItem value="24h">Last 24 Hours</MenuItem>
                      <MenuItem value="7d">Last 7 Days</MenuItem>
                      <MenuItem value="30d">Last 30 Days</MenuItem>
                    </Select>
                  </FormControl>
                </Grid>

                <Grid xs={12} sm={6} md={3}>
                  <Button
                    fullWidth
                    variant="contained"
                    onClick={fetchMetrics}
                    disabled={isLoading || !entityId.trim()}
                    startIcon={<ArrowClockwiseIcon />}
                  >
                    {isLoading ? 'Loading...' : 'Fetch Metrics'}
                  </Button>
                </Grid>
              </Grid>
            </CardContent>
          </Card>

          {/* Loading state */}
          {isLoading && (
            <Box sx={{ display: 'flex', justifyContent: 'center', mt: 4 }}>
              <CircularProgress />
            </Box>
          )}

          {/* Error state */}
          {error && <Alert severity="error">Error: {error.message}</Alert>}

          {/* Metrics data */}
          {metricsData && !isLoading && !error && (
            <React.Fragment>
              <Typography variant="h5">
                {metricType === 'organization' ? 'Organization' : 'Chama'}{' '}
                Metrics
              </Typography>

              <Grid container spacing={3}>
                {metricType === 'organization' ? (
                  // Organization metrics
                  <>
                    <Grid xs={12} md={6}>
                      {renderMetricCard('Members', metricsData.members)}
                    </Grid>
                    <Grid xs={12} md={6}>
                      {renderMetricCard('Financial', metricsData.financial)}
                    </Grid>
                    <Grid xs={12} md={6}>
                      {renderMetricCard('Activity', metricsData.activity)}
                    </Grid>
                    <Grid xs={12} md={6}>
                      {renderMetricCard('Compliance', metricsData.compliance)}
                    </Grid>
                  </>
                ) : (
                  // Chama metrics
                  <>
                    <Grid xs={12} md={6}>
                      {renderMetricCard('Members', metricsData.members)}
                    </Grid>
                    <Grid xs={12} md={6}>
                      {renderMetricCard(
                        'Contributions',
                        metricsData.contributions,
                      )}
                    </Grid>
                    <Grid xs={12} md={6}>
                      {renderMetricCard('Loans', metricsData.loans)}
                    </Grid>
                    <Grid xs={12} md={6}>
                      {renderMetricCard('Activity', metricsData.activity)}
                    </Grid>
                  </>
                )}
              </Grid>

              {/* Raw data display */}
              <Card>
                <CardHeader title="Raw Data" />
                <CardContent>
                  <Box
                    component="pre"
                    sx={{
                      backgroundColor: 'grey.100',
                      p: 2,
                      borderRadius: 1,
                      overflow: 'auto',
                      maxHeight: 400,
                      fontSize: '0.875rem',
                      fontFamily: 'monospace',
                    }}
                  >
                    {JSON.stringify(metricsData, null, 2)}
                  </Box>
                </CardContent>
              </Card>
            </React.Fragment>
          )}

          {/* Instructions */}
          {!metricsData && !isLoading && !error && (
            <Card>
              <CardContent>
                <Typography variant="body1" gutterBottom>
                  Instructions:
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  1. Select whether you want to view Organization or Chama
                  metrics
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  2. Enter the ID of the organization or chama
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  3. Choose a time range for the metrics
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  4. Click "Fetch Metrics" to view the data
                </Typography>
              </CardContent>
            </Card>
          )}
        </Stack>
      </Container>
    </Box>
  );
}
