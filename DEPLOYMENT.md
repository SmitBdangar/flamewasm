# Deployment Guide

This guide covers deployment options for the Data Visualization Dashboard (MERN Stack).

## Recommended Deployment Strategy

### Option 1: Easy & Free (Recommended for Testing)
- **Frontend**: Vercel or Netlify (Free tier)
- **Backend**: Railway or Render (Free tier available)
- **Database**: MongoDB Atlas (Free tier - 512MB)

### Option 2: Production Ready
- **Frontend**: Vercel or Netlify
- **Backend**: Railway, Render, or Heroku
- **Database**: MongoDB Atlas

### Option 3: Full Control
- **Frontend**: AWS S3 + CloudFront or DigitalOcean App Platform
- **Backend**: AWS EC2, DigitalOcean Droplet, or Google Cloud Run
- **Database**: MongoDB Atlas or self-hosted MongoDB

---

## Deployment Steps

### Step 1: Set Up MongoDB Atlas (Cloud Database)

1. **Create MongoDB Atlas Account**
   - Go to https://www.mongodb.com/cloud/atlas
   - Sign up for free account
   - Create a new cluster (Free tier: M0)

2. **Configure Database Access**
   - Go to "Database Access"
   - Create a database user (save username/password)
   - Set network access to allow from anywhere (0.0.0.0/0) or specific IPs

3. **Get Connection String**
   - Go to "Database" → "Connect"
   - Choose "Connect your application"
   - Copy the connection string
   - Replace `<password>` with your database user password
   - Example: `mongodb+srv://username:password@cluster0.xxxxx.mongodb.net/visualization_dashboard?retryWrites=true&w=majority`

---

## Option A: Deploy to Vercel + Railway (Easiest)

### Deploy Backend to Railway

1. **Prepare Backend**
   ```bash
   cd backend
   ```

2. **Create Railway Account**
   - Go to https://railway.app
   - Sign up with GitHub

3. **Deploy Backend**
   - Click "New Project"
   - Select "Deploy from GitHub repo"
   - Connect your repository
   - Select the `backend` folder
   - Add environment variables:
     ```
     PORT=5000
     MONGODB_URI=your_mongodb_atlas_connection_string
     NODE_ENV=production
     ```
   - Railway will auto-detect Node.js and deploy

4. **Get Backend URL**
   - Railway provides a URL like: `https://your-app.railway.app`
   - Note this URL for frontend configuration

### Deploy Frontend to Vercel

1. **Update Frontend API URL**
   - Create `.env.production` in `frontend/`:
     ```
     REACT_APP_API_URL=https://your-backend.railway.app/api
     ```

2. **Deploy to Vercel**
   - Go to https://vercel.com
   - Sign up with GitHub
   - Click "New Project"
   - Import your repository
   - Set root directory to `frontend`
   - Add environment variable:
     ```
     REACT_APP_API_URL=https://your-backend.railway.app/api
     ```
   - Deploy

3. **Seed Database**
   - SSH into Railway or use Railway CLI:
     ```bash
     npm install -g @railway/cli
     railway login
     railway run npm run seed
     ```

---

## Option B: Deploy to Render (Alternative)

### Deploy Backend to Render

1. **Create Render Account**
   - Go to https://render.com
   - Sign up with GitHub

2. **Create Web Service**
   - Click "New" → "Web Service"
   - Connect your repository
   - Settings:
     - **Name**: visualization-dashboard-backend
     - **Root Directory**: backend
     - **Environment**: Node
     - **Build Command**: `npm install`
     - **Start Command**: `npm start`
   - Add environment variables:
     ```
     PORT=5000
     MONGODB_URI=your_mongodb_atlas_connection_string
     NODE_ENV=production
     ```
   - Click "Create Web Service"

### Deploy Frontend to Netlify

1. **Update Frontend API URL**
   - Create `netlify.toml` in `frontend/`:
     ```toml
     [build]
       command = "npm run build"
       publish = "build"
     
     [[redirects]]
       from = "/api/*"
       to = "https://your-backend.onrender.com/api/:splat"
       status = 200
     ```

2. **Deploy to Netlify**
   - Go to https://netlify.com
   - Sign up with GitHub
   - Click "New site from Git"
   - Connect repository
   - Settings:
     - **Base directory**: frontend
     - **Build command**: `npm run build`
     - **Publish directory**: `frontend/build`
   - Add environment variable:
     ```
     REACT_APP_API_URL=https://your-backend.onrender.com/api
     ```
   - Deploy

---

## Option C: Deploy to Heroku (Classic Option)

### Deploy Backend to Heroku

1. **Install Heroku CLI**
   ```bash
   # Windows: Download from https://devcenter.heroku.com/articles/heroku-cli
   # Mac: brew install heroku/brew/heroku
   # Linux: sudo snap install heroku --classic
   ```

2. **Create Heroku App**
   ```bash
   cd backend
   heroku login
   heroku create your-app-name-backend
   ```

3. **Configure Environment Variables**
   ```bash
   heroku config:set MONGODB_URI=your_mongodb_atlas_connection_string
   heroku config:set NODE_ENV=production
   ```

4. **Deploy**
   ```bash
   git add .
   git commit -m "Deploy backend"
   git push heroku main
   ```

5. **Seed Database**
   ```bash
   heroku run npm run seed
   ```

### Deploy Frontend to Heroku

1. **Add Buildpack**
   ```bash
   cd frontend
   heroku create your-app-name-frontend
   heroku buildpacks:set mars/create-react-app
   ```

2. **Configure Environment**
   ```bash
   heroku config:set REACT_APP_API_URL=https://your-backend.herokuapp.com/api
   ```

3. **Deploy**
   ```bash
   git add .
   git commit -m "Deploy frontend"
   git push heroku main
   ```

---

## Option D: Deploy Both to Vercel (Serverless)

### Backend on Vercel

1. **Create `vercel.json` in backend/**
   ```json
   {
     "version": 2,
     "builds": [
       {
         "src": "server.js",
         "use": "@vercel/node"
       }
     ],
     "routes": [
       {
         "src": "/(.*)",
         "dest": "server.js"
       }
     ]
   }
   ```

2. **Deploy**
   - Connect repo to Vercel
   - Set root directory to `backend`
   - Add environment variables
   - Deploy

### Frontend on Vercel
- Same as Option A

---

## Important Configuration Files

### Backend: Update CORS for Production

Update `backend/server.js` to allow your frontend domain:

```javascript
const corsOptions = {
  origin: process.env.FRONTEND_URL || 'http://localhost:3000',
  credentials: true
};
app.use(cors(corsOptions));
```

### Frontend: Environment Variables

Create `frontend/.env.production`:
```
REACT_APP_API_URL=https://your-backend-url.com/api
```

### Backend: Environment Variables

Create `backend/.env` (or set in hosting platform):
```
PORT=5000
MONGODB_URI=mongodb+srv://username:password@cluster.mongodb.net/dbname
NODE_ENV=production
FRONTEND_URL=https://your-frontend-url.com
```

---

## Post-Deployment Checklist

- [ ] MongoDB Atlas cluster is running
- [ ] Database is seeded with data
- [ ] Backend API is accessible
- [ ] Frontend can connect to backend
- [ ] CORS is configured correctly
- [ ] Environment variables are set
- [ ] SSL/HTTPS is enabled (most platforms do this automatically)

---

## Troubleshooting

### CORS Errors
- Ensure backend CORS allows your frontend domain
- Check environment variables are set correctly

### Database Connection Issues
- Verify MongoDB Atlas IP whitelist includes 0.0.0.0/0
- Check connection string has correct password
- Ensure database user has read/write permissions

### Build Failures
- Check Node.js version compatibility
- Verify all dependencies are in package.json
- Check build logs for specific errors

### API Not Responding
- Verify backend is running
- Check backend logs
- Ensure PORT environment variable is set
- Verify API routes are correct

---

## Cost Comparison

| Platform | Free Tier | Paid Tier |
|----------|-----------|-----------|
| **Vercel** | ✅ Unlimited (with limits) | $20/month |
| **Netlify** | ✅ 100GB bandwidth | $19/month |
| **Railway** | ✅ $5 credit/month | $5+/month |
| **Render** | ✅ Free tier (sleeps after inactivity) | $7+/month |
| **Heroku** | ❌ No free tier | $7+/month |
| **MongoDB Atlas** | ✅ 512MB free | $9+/month |

---

## Recommended for Different Use Cases

### **Personal Project / Portfolio**
→ Vercel (Frontend) + Railway (Backend) + MongoDB Atlas (Free)

### **Small Business / Startup**
→ Netlify (Frontend) + Render (Backend) + MongoDB Atlas

### **Enterprise / High Traffic**
→ AWS (S3 + CloudFront + EC2/ECS) + MongoDB Atlas

### **Quick Prototype**
→ Vercel (Both Frontend & Backend) + MongoDB Atlas

---

## Quick Deploy Commands

### Railway (Backend)
```bash
npm install -g @railway/cli
railway login
railway init
railway up
railway run npm run seed
```

### Vercel (Frontend)
```bash
npm install -g vercel
cd frontend
vercel --prod
```

### Netlify (Frontend)
```bash
npm install -g netlify-cli
cd frontend
netlify deploy --prod
```

